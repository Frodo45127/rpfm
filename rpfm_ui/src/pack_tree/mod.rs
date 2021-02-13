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
This module contains code to make our live easier when dealing with `TreeViews`.
!*/

use qt_widgets::QTreeView;
use qt_widgets::q_header_view::ResizeMode;

use qt_gui::QBrush;
use qt_gui::QColor;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;
use qt_gui::QListOfQStandardItem;

use qt_core::QBox;
use qt_core::ItemFlag;
use qt_core::QFlags;
use qt_core::QModelIndex;
use qt_core::GlobalColor;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QVariant;
use qt_core::QPtr;
use qt_core::QSignalBlocker;

use cpp_core::CppBox;
use cpp_core::Ptr;
use cpp_core::Ref;
use cpp_core::CastFrom;

use chrono::naive::NaiveDateTime;
use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};

use std::cmp::Ordering;
use std::path::PathBuf;
use std::rc::Rc;

use rpfm_lib::common::get_files_from_subdir;
use rpfm_lib::packfile::packedfile::PackedFileInfo;
use rpfm_lib::packfile::{CompressionState, PackFileInfo, PathType, PFHFlags};
use rpfm_lib::SETTINGS;
use rpfm_lib::SUPPORTED_GAMES;

use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::pack_tree::icons::IconType;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::{
    YELLOW_BRIGHT,
    YELLOW_MEDIUM,
    YELLOW_DARK,
    GREEN_BRIGHT,
    GREEN_MEDIUM,
    GREEN_DARK,
    RED_BRIGHT,
    RED_DARK,
    MAGENTA_MEDIUM,
    MEDIUM_DARKER_GREY,
    TRANSPARENT_BRIGHT,
    ERROR_UNPRESSED_DARK,
    ERROR_UNPRESSED_LIGHT,
    ERROR_PRESSED_DARK,
    ERROR_PRESSED_LIGHT,
    ERROR_FOREGROUND_LIGHT,
    WARNING_UNPRESSED_DARK,
    WARNING_UNPRESSED_LIGHT,
    WARNING_PRESSED_DARK,
    WARNING_PRESSED_LIGHT,
    WARNING_FOREGROUND_LIGHT,
    INFO_UNPRESSED_DARK,
    INFO_UNPRESSED_LIGHT,
    INFO_PRESSED_DARK,
    INFO_PRESSED_LIGHT
};

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

// Used to specify that a PackedFile inside it has been deleted. Unused for now.
//const ITEM_STATUS_DELETED: i32 = 4;

//-------------------------------------------------------------------------------//
//                          Enums & Structs (and trait)
//-------------------------------------------------------------------------------//

/// This trait adds multiple util functions to the `TreeView` you implement it for.
///
/// Keep in mind that this trait has been created with `PackFile TreeView's` in mind, so his methods
/// may not be suitable for all purposes.
pub trait PackTree {

    /// This function allows us to add the provided item into the path we want on the `TreeView`, taking care of adding missing parents.
    ///
    /// The way this function works is by replacing the destination path of the UI, if exists, so be careful with that..
    unsafe fn add_row_to_path(item: Ptr<QListOfQStandardItem>, model: &QPtr<QStandardItemModel>, path: &[String], packed_file_info: &Option<PackedFileInfo>);

    /// This function is used to expand the entire path from the PackFile to an specific item in the `TreeView`.
    ///
    /// It returns the `ModelIndex` of the final item of the path, or None if it wasn't found or it's hidden by the filter.
    unsafe fn expand_treeview_to_item(&self, path: &[String]) -> Option<Ptr<QModelIndex>>;

    /// This function is used to expand an item and all it's children recursively.
    unsafe fn expand_all_from_item(tree_view: &QTreeView, item: Ptr<QStandardItem>, first_item: bool);

    /// This function is used to expand an item and all it's children recursively.
    unsafe fn expand_all_from_type(tree_view: &QTreeView, item: &TreePathType);

    /// This function gives you the items selected in the PackFile Content's TreeView.
    unsafe fn get_items_from_main_treeview_selection(pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Vec<Ptr<QStandardItem>>;

    /// This function gives you the items selected in the provided `TreeView`.
    unsafe fn get_items_from_selection(&self, has_filter: bool) -> Vec<Ptr<QStandardItem>>;

    /// This function gives you the `TreeViewTypes` of the items selected in the PackFile Content's TreeView.
    unsafe fn get_item_types_from_main_treeview_selection(pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Vec<TreePathType>;

    /// This function gives you the `TreeViewTypes` of the items selected in the provided.
    unsafe fn get_item_types_from_selection(&self, has_filter: bool) -> Vec<TreePathType>;

    /// This function returns the `TreePathType`s not hidden by the applied filter corresponding to the current selection.
    ///
    /// This always assumes the `TreeView` has a filter. It'll die horrendously otherwise.
    unsafe fn get_item_types_from_selection_filtered(&self) -> Vec<TreePathType>;

    /// This function gives you the item corresponding to an specific `TreePathType`.
    unsafe fn get_item_from_type(item_type: &TreePathType, model: &QPtr<QStandardItemModel>) -> Ptr<QStandardItem>;

    /// This function gives you a bitmask with what's selected in the PackFile Content's TreeView,
    /// the number of selected files, and the number of selected folders.
    unsafe fn get_combination_from_main_treeview_selection(pack_file_contents_ui: &Rc<PackFileContentsUI>) -> (u8, u32, u32);

    /// This function returns the `TreePathType` of the provided item. Unsafe version.
    unsafe fn get_type_from_item(item: Ptr<QStandardItem>, model: &QPtr<QStandardItemModel>) -> TreePathType;

    /// This function is used to get the path of a specific Item in a StandardItemModel. Unsafe version.
    unsafe fn get_path_from_item(item: Ptr<QStandardItem>, model: &QPtr<QStandardItemModel>) -> Vec<String>;

    /// This function is used to get the path of a specific ModelIndex in a StandardItemModel. Unsafe version.
    unsafe fn get_path_from_index(index: Ref<QModelIndex>, model: &QPtr<QStandardItemModel>) -> Vec<String>;

    /// This function gives you the path of the items selected in the PackFile Content's TreeView.
    unsafe fn get_path_from_main_treeview_selection(pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Vec<Vec<String>>;

    /// This function gives you the path it'll have in the PackFile Content's TreeView a file from disk.
    unsafe fn get_path_from_pathbuf(pack_file_contents_ui: &Rc<PackFileContentsUI>, file_path: &PathBuf, is_file: bool) -> Vec<Vec<String>>;

    /// This function changes the color of an specific item from the PackFile Content's TreeView according to his current state.
    unsafe fn paint_specific_item_treeview(item: Ptr<QStandardItem>);

    /// This function removes the item under the provided path and returns it.
    unsafe fn take_row_from_type(item_type: &TreePathType, model: &QPtr<QStandardItemModel>) -> Ptr<QListOfQStandardItem>;

    /// This function takes care of EVERY operation that manipulates the provided TreeView.
    /// It does one thing or another, depending on the operation we provide it.
    ///
    /// BIG NOTE: Each StandardItem should keep track of his own status, meaning that their data means:
    /// - Position 20: Type. 1 is File, 2 is Folder, 4 is PackFile.
    /// - Position 21: Status. 0 is untouched, 1 is added, 2 is modified, 3 is added + modified.
    /// In case you don't realise, those are bitmasks.
    unsafe fn update_treeview(&self, has_filter: bool, operation: TreeViewOperation);
}

/// This enum has the different possible operations we can do in a `TreeView`.
#[derive(Clone, Debug)]
pub enum TreeViewOperation {

    /// Build the entire `TreeView` from nothing. Requires an option: Some<PathBuf> if the `PackFile` is not editable, `None` if it is.
    /// Also, you can pass a PackFileInfo/PackedFileInfo if you want to build a TreeView with custom data.
    Build(Option<PathBuf>, Option<(PackFileInfo, Vec<PackedFileInfo>)>),

    /// Add one or more files/folders to the `TreeView`. Requires a `Vec<TreePathType>` to add to the `TreeView`.
    Add(Vec<TreePathType>),

    /// Remove the files/folders corresponding to the `Vec<TreePathType>` we provide from the `TreeView`.
    Delete(Vec<TreePathType>),

    /// Set the provided paths as *modified*. It requires the `Vec<TreePathType>` of whatever you want to mark as modified.
    Modify(Vec<TreePathType>),

    /// Change the name of a file/folder from the `TreeView`. Requires the `TreePathType` of whatever you want to move, and its new name.
    Move(Vec<(TreePathType, Vec<String>)>),

    /// Mark an item as ***Always Modified*** so it cannot be marked as unmodified by an undo operation.
    MarkAlwaysModified(Vec<TreePathType>),

    /// Resets the state of one or more `TreePathType` to 0, or unmodified.
    Undo(Vec<TreePathType>),

    /// Remove all status and color from the entire `TreeView`.
    Clean,

    /// Remove all items from the `TreeView`.
    Clear,

    /// Updates the tooltip of the PackedFiles with the provided info.
    UpdateTooltip(Vec<PackedFileInfo>),
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
        matches!((self, other),
            (&TreePathType::File(_), &TreePathType::File(_)) |
            (&TreePathType::Folder(_), &TreePathType::Folder(_)) |
            (&TreePathType::PackFile, &TreePathType::PackFile) |
            (&TreePathType::None, &TreePathType::None))
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

/// Implementation of `PackTree` for `QPtr<QTreeView>.
impl PackTree for QBox<QTreeView> {

    unsafe fn add_row_to_path(row: Ptr<QListOfQStandardItem>, model: &QPtr<QStandardItemModel>, path: &[String], packed_file_info: &Option<PackedFileInfo>) {

        // First, we go down the tree to the row we have to take.
        let type_to_skip = if row.value_1a(0).data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { ITEM_TYPE_FOLDER } else { ITEM_TYPE_FILE };
        let mut item = model.item_1a(0);
        let item_status = get_status_item_from_item(item);
        item_status.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS);
        item_status.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);

        // First looping downwards. -1 because we want to reach the "parent" that will hold the new row, not the row itself.
        for path_item in path {
            for row in 0..item.row_count() {
                let child = item.child_1a(row);

                // We are only interested in folders.
                if child.data_1a(ITEM_TYPE).to_int_0a() != ITEM_TYPE_FOLDER { continue }

                // If we found it, we're done.
                if child.text().to_std_string() == *path_item {

                    let child_status = get_status_item_from_item(child);
                    child_status.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS);
                    child_status.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                    item = child;
                    break;
                }
            }
        }

        // Now, update the row and add it.
        if let Some(info) = packed_file_info {
            let tooltip = new_packed_file_tooltip(&info);
            row.value_1a(0).set_tool_tip(&QString::from_std_str(tooltip));
        }

        // If there was an item with than name, remove it.
        for row in 0..item.row_count() {
            let child = item.child_1a(row);
            if child.data_1a(ITEM_TYPE).to_int_0a() == type_to_skip { continue }
            if child.text().to_std_string() == *path.last().unwrap() {
                item.remove_row(row);
                break;
            }
        }

        item.append_row_q_list_of_q_standard_item(row.as_ref().unwrap());

        row.value_1a(0).set_text(&QString::from_std_str(path.last().unwrap()));
        row.value_1a(1).set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS);
        row.value_1a(1).set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);

        // Sort the TreeView by its proper type.
        if type_to_skip == ITEM_TYPE_FILE {
            sort_item_in_tree_view(
                &model,
                row.value_1a(0),
                &TreePathType::Folder(vec![String::new()])
            );
        }

        else {
            sort_item_in_tree_view(
                &model,
                row.value_1a(0),
                &TreePathType::File(vec![String::new()])
            );
        }
    }

    unsafe fn expand_treeview_to_item(&self, path: &[String]) -> Option<Ptr<QModelIndex>> {
        let filter: QPtr<QSortFilterProxyModel> = self.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();

        // Get the first item's index, as that one should always exist (the Packfile).
        let mut item = model.item_1a(0);
        let model_index = model.index_2a(0, 0);
        let filtered_index = filter.map_from_source(&model_index);

        // If it's valid (filter didn't hid it away), we expand it and search among its children the next one to expand.
        if filtered_index.is_valid() {
            self.expand(&filtered_index);

            // Indexes to see how deep we must go.
            let mut index = 0;
            let path_deep = path.len();
            if path_deep > 0 {

                loop {

                    let mut not_found = true;
                    for row in 0..item.row_count() {
                        let child = item.child_1a(row);

                        // In the last cycle, we're interested in files, not folders.
                        if index == (path_deep -1) {

                            if child.has_children() { continue; }

                            // We guarantee that the name of the files/folders is unique, so we use it to find the one to expand.
                            if path[index] == child.text().to_std_string() {
                                item = child;

                                let model_index = model.index_from_item(item);
                                let filtered_index = filter.map_from_source(&model_index);

                                if filtered_index.is_valid() { return Some(filtered_index.into_ptr()); }
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
                                let model_index = model.index_from_item(item);
                                let filtered_index = filter.map_from_source(&model_index);

                                if filtered_index.is_valid() { self.expand(&filtered_index); }
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
        }
        None
    }

    unsafe fn expand_all_from_type(tree_view: &QTreeView, item: &TreePathType) {
        let filter: QPtr<QSortFilterProxyModel> = tree_view.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
        let item = Self::get_item_from_type(item, &model);
        Self::expand_all_from_item(tree_view, item, true);
    }

    unsafe fn expand_all_from_item(tree_view: &QTreeView, item: Ptr<QStandardItem>, first_item: bool) {
        let filter: QPtr<QSortFilterProxyModel> = tree_view.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();

        // First, expand our item, then expand its children.
        let model_index = model.index_from_item(item);
        if first_item {
            let filtered_index = filter.map_from_source(&model_index);
            if filtered_index.is_valid() {
                tree_view.expand(&filtered_index);
            }
        }
        for row in 0..item.row_count() {
            let child = item.child_1a(row);
            if child.has_children() {
                let model_index = model.index_from_item(item);
                let filtered_index = filter.map_from_source(&model_index);
                if filtered_index.is_valid() {
                    tree_view.expand(&filtered_index);
                    Self::expand_all_from_item(tree_view, Ptr::cast_from(child), false);
                }
            }
        }
    }

    unsafe fn get_items_from_main_treeview_selection(pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Vec<Ptr<QStandardItem>> {
        let tree_view = &pack_file_contents_ui.packfile_contents_tree_view;
        let filter: QPtr<QSortFilterProxyModel> = tree_view.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();

        let indexes_visual = tree_view.selection_model().selection().indexes();
        let indexes_visual = (0..indexes_visual.count_0a()).map(|x| indexes_visual.at(x)).collect::<Vec<Ref<QModelIndex>>>();
        let indexes_real = indexes_visual.iter().map(|x| filter.map_to_source(*x)).collect::<Vec<CppBox<QModelIndex>>>();
        indexes_real.iter().map(|x| model.item_from_index(x)).collect()
    }

    unsafe fn get_items_from_selection(&self, has_filter: bool) -> Vec<Ptr<QStandardItem>> {
        let filter: Option<QPtr<QSortFilterProxyModel>> = if has_filter { Some(self.model().static_downcast()) } else { None };
        let model: QPtr<QStandardItemModel> = if let Some(ref filter) = filter { filter.source_model().static_downcast() } else { self.model().static_downcast()};

        let indexes_visual = self.selection_model().selection().indexes();
        let mut indexes_visual = (0..indexes_visual.count_0a()).rev().map(|x| indexes_visual.take_at(x)).collect::<Vec<CppBox<QModelIndex>>>();
        indexes_visual.reverse();
        let indexes_real = if let Some(filter) = filter {
            indexes_visual.iter().map(|x| filter.map_to_source(x.as_ref())).collect::<Vec<CppBox<QModelIndex>>>()
        } else {
            indexes_visual
        };

        indexes_real.iter().map(|x| model.item_from_index(x.as_ref())).collect()
    }

    unsafe fn get_item_types_from_main_treeview_selection(pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Vec<TreePathType> {
        let items = Self::get_items_from_main_treeview_selection(pack_file_contents_ui);
        items.iter().map(|x| Self::get_type_from_item(*x, &pack_file_contents_ui.packfile_contents_tree_model.static_upcast())).collect()
    }

    unsafe fn get_item_types_from_selection(&self, has_filter: bool) -> Vec<TreePathType> {
        let items = self.get_items_from_selection(has_filter);

        let model: QPtr<QStandardItemModel> = if has_filter {
            let filter: QPtr<QSortFilterProxyModel> = self.model().static_downcast();
            filter.source_model().static_downcast()
        } else {
            self.model().static_downcast()
        };

        items.iter().map(|x| Self::get_type_from_item(*x, &model)).collect()
    }

    unsafe fn get_item_types_from_selection_filtered(&self)-> Vec<TreePathType> {
        let filter: QPtr<QSortFilterProxyModel> = self.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();

        let mut item_types = vec![];
        let item_types_selected = self.get_item_types_from_selection(true);
        for item_type in &item_types_selected {
            match item_type {
                 TreePathType::File(_) => item_types.push(item_type.clone()),
                 TreePathType::Folder(_) | TreePathType::PackFile => {
                    let item = <QBox<QTreeView> as PackTree>::get_item_from_type(&item_type, &model);
                    get_visible_childs_of_item(&item, self, &filter, &model, &mut item_types);
                 }
                 TreePathType::None => unreachable!(),
            }
        }

        item_types
    }

    unsafe fn get_item_from_type(item_type: &TreePathType, model: &QPtr<QStandardItemModel>) -> Ptr<QStandardItem> {

        // Get it another time, this time to use it to hold the current item.
        let mut item = model.item_1a(0);
        match item_type {
            TreePathType::File(ref path) | TreePathType::Folder(ref path) => {
                let mut index = 0;
                let path_deep = path.len();
                loop {

                    // If we reached the folder of the item...
                    if index == (path_deep - 1) {
                        let children_count = item.row_count();
                        for row in 0..children_count {
                            let child = item.child_1a(row);

                            // We ignore files or folders, depending on what we want to create.
                            if let TreePathType::File(_) = &item_type {
                                if child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FOLDER { continue }
                            }

                            if let TreePathType::Folder(_) = &item_type {
                                if child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { continue }
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
                            let child = item.child_1a(row);
                            if child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { continue }

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

    unsafe fn get_combination_from_main_treeview_selection(pack_file_contents_ui: &Rc<PackFileContentsUI>) -> (u8, u32, u32) {

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

    unsafe fn get_type_from_item(item: Ptr<QStandardItem>, model: &QPtr<QStandardItemModel>) -> TreePathType {
        match item.data_1a(ITEM_TYPE).to_int_0a() {
            0 => TreePathType::None,
            ITEM_TYPE_FILE => TreePathType::File(Self::get_path_from_item(item, &model)),
            ITEM_TYPE_FOLDER => TreePathType::Folder(Self::get_path_from_item(item, &model)),
            ITEM_TYPE_PACKFILE => TreePathType::PackFile,
            _ => unimplemented!()
        }
    }

    unsafe fn get_path_from_item(item: Ptr<QStandardItem>, model: &QPtr<QStandardItemModel>) -> Vec<String> {
        let index = item.index();
        Self::get_path_from_index(index.as_ref(), &model)
    }

    unsafe fn get_path_from_index(index: Ref<QModelIndex>, model: &QPtr<QStandardItemModel>) -> Vec<String> {

        // The logic is simple: we loop from item to parent until we reach the top.
        let mut path = vec![];
        let mut index = index;
        let mut parent;

        // Loop until we reach the root index.
        loop {
            let text = model.data_1a(index).to_string().to_std_string();
            parent = index.parent();

            // If the parent is valid, it's the new item. Otherwise, we stop without adding it (we don't want the PackFile's name in).
            if parent.is_valid() {
                path.push(text);
                index = parent.as_ref();
            } else { break; }
        }

        // Reverse it, as we want it from arent to children.
        path.reverse();
        path
    }

    unsafe fn get_path_from_main_treeview_selection(pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Vec<Vec<String>> {

        // Create the vector to hold the Paths and get the selected indexes of the TreeView.
        let tree_view = &pack_file_contents_ui.packfile_contents_tree_view;
        let filter: QPtr<QSortFilterProxyModel> = tree_view.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
        let selection_model = tree_view.selection_model();

        let mut paths: Vec<Vec<String>> = vec![];
        let indexes = filter.map_selection_to_source(&selection_model.selection()).indexes();
        for index_num in 0..indexes.count_0a() {
            paths.push(Self::get_path_from_index(indexes.at(index_num), &model));
        }
        paths
    }

    unsafe fn get_path_from_pathbuf(pack_file_contents_ui: &Rc<PackFileContentsUI>, file_path: &PathBuf, is_file: bool) -> Vec<Vec<String>> {
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

    unsafe fn paint_specific_item_treeview(item: Ptr<QStandardItem>) {
        let color_added = get_color_added_high_intensity();
        let color_modified = get_color_modified_high_intensity();
        let color_added_modified = get_color_added_modified_high_intensity();
        let color_untouched = get_color_unmodified();
        match item.data_1a(ITEM_STATUS).to_int_0a() {
            ITEM_STATUS_PRISTINE => item.set_background(&QBrush::from_q_color(color_untouched.as_ref().unwrap())),
            ITEM_STATUS_ADDED => item.set_background(&QBrush::from_q_color(color_added.as_ref().unwrap())),
            ITEM_STATUS_MODIFIED => item.set_background(&QBrush::from_q_color(color_modified.as_ref().unwrap())),
            3 => item.set_background(&QBrush::from_q_color(color_added_modified.as_ref().unwrap())),
            _=> unimplemented!(),
        };
    }

    unsafe fn take_row_from_type(item_type: &TreePathType, model: &QPtr<QStandardItemModel>) -> Ptr<QListOfQStandardItem> {
        match item_type {

            // Different types require different methods...
            TreePathType::File(path) | TreePathType::Folder(path) => {

                // First, we go down the tree to the row we have to take.
                let is_file = matches!(item_type, TreePathType::File(_));
                let mut item = model.item_1a(0);
                let item_status = get_status_item_from_item(item);
                item_status.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS);
                item_status.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);

                // First looping downwards.
                for index in 0..path.len() {
                    for row in 0..item.row_count() {
                        let child = item.child_1a(row);

                        // If we have not yet reach the last item of our path, check only folders.
                        if index < path.len() - 1 {
                            if child.data_1a(ITEM_TYPE).to_int_0a() != ITEM_TYPE_FOLDER { continue }
                        }

                        // If we have reached the last item, ignore anything not of the intended type.
                        else {
                            if is_file && child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FOLDER { continue }
                            if !is_file && child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { continue }
                        }

                        // If we found it, we're done.
                        if child.text().to_std_string() == path[index] {

                            let child_status = get_status_item_from_item(child);
                            child_status.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS);
                            child_status.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                            item = child;
                            break;
                        }
                    }
                }

                // Take the child.
                item.parent().take_row(item.row()).into_ptr()
            }

            _ => unimplemented!()
        }
    }

    unsafe fn update_treeview(&self, has_filter: bool, operation: TreeViewOperation) {
        let filter: Option<QPtr<QSortFilterProxyModel>> = if has_filter { Some(self.model().static_downcast()) } else { None };
        let model: QPtr<QStandardItemModel> = if let Some(ref filter) = filter { filter.source_model().static_downcast() } else { self.model().static_downcast() };

        // We act depending on the operation requested.
        match operation {

            // If we want to build a new TreeView...
            TreeViewOperation::Build(extra_packfile_path, internal_packed_file) => {

                // Depending on the PackFile we want to build the TreeView with, we ask for his data. Internal files take priority.
                let (pack_file_data, packed_files_data) = if let Some(data) = internal_packed_file {
                    data
                }
                else if let Some(ref path) = extra_packfile_path {
                    CENTRAL_COMMAND.send_message_qt(Command::GetPackFileExtraDataForTreeView(path.to_path_buf()));
                    let response = CENTRAL_COMMAND.recv_message_qt();
                    if let Response::PackFileInfoVecPackedFileInfo(data) = response { data }
                    else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); }
                }
                else {
                    CENTRAL_COMMAND.send_message_qt(Command::GetPackFileDataForTreeView);
                    let response = CENTRAL_COMMAND.recv_message_qt();
                    if let Response::PackFileInfoVecPackedFileInfo(data) = response { data }
                    else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response) }
                };
                let mut sorted_path_list = packed_files_data;

                // First, we clean the TreeStore and whatever was created in the TreeView.
                model.clear();

                // Second, we set as the big_parent, the base for the folders of the TreeView, a fake folder
                // with the name of the PackFile. All big things start with a lie.
                let big_parent = QStandardItem::from_q_string(&QString::from_std_str(&pack_file_data.file_name));
                let state_item = QStandardItem::new();
                let tooltip = new_pack_file_tooltip(&pack_file_data);
                big_parent.set_tool_tip(&QString::from_std_str(tooltip));
                big_parent.set_editable(false);
                big_parent.set_data_2a(&QVariant::from_int(ITEM_TYPE_PACKFILE), ITEM_TYPE);
                state_item.set_data_2a(&QVariant::from_int(ITEM_STATUS_PRISTINE), ITEM_STATUS);
                state_item.set_editable(false);
                let flags = ItemFlag::from(state_item.flags().to_int() & ItemFlag::ItemIsSelectable.to_int());
                state_item.set_flags(QFlags::from(flags));
                let icon_type = IconType::PackFile(extra_packfile_path.is_some());
                icon_type.set_icon_to_item_safe(&big_parent);

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

                            // Otherwise, it means you got 2 files with the same name in the same PackFile, and I would like to know how the hell did you did it.
                            else {
                                return Ordering::Equal
                            }
                        }

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
                    let mut parent = big_parent.as_ptr();
                    for (index_in_path, name) in packed_file.path.iter().enumerate() {

                        // If it's the last string in the file path, it's a file, so we add it to the model.
                        if index_in_path == packed_file.path.len() - 1 {
                            let file = QStandardItem::from_q_string(&QString::from_std_str(name));
                            let state_item = QStandardItem::new();
                            let tooltip = new_packed_file_tooltip(&packed_file);
                            file.set_tool_tip(&QString::from_std_str(tooltip));
                            file.set_editable(false);
                            file.set_data_2a(&QVariant::from_int(ITEM_TYPE_FILE), ITEM_TYPE);
                            state_item.set_data_2a(&QVariant::from_int(ITEM_STATUS_PRISTINE), ITEM_STATUS);
                            state_item.set_editable(false);
                            state_item.set_flags(QFlags::from(flags));
                            let icon_type = IconType::File(packed_file.path.to_vec());
                            icon_type.set_icon_to_item_safe(&file);

                            let qlist = QListOfQStandardItem::new();
                            qlist.append_q_standard_item(&file.into_ptr().as_mut_raw_ptr());
                            qlist.append_q_standard_item(&state_item.into_ptr().as_mut_raw_ptr());

                            parent.append_row_q_list_of_q_standard_item(qlist.as_ref());
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
                                    let child = parent.child_2a(index, 0);
                                    if child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { continue }

                                    // Get his text. If it's the same folder we are trying to add, this is our parent now.
                                    if child.text().to_std_string() == *name {
                                        parent = parent.child_1a(index);
                                        duplicate_found = true;
                                        break;
                                    }
                                }
                            }

                            // If our current parent doesn't have anything, just add it as a new folder.
                            if !duplicate_found {
                                let folder = QStandardItem::from_q_string(&QString::from_std_str(name));
                                let state_item = QStandardItem::new();
                                folder.set_editable(false);
                                folder.set_data_2a(&QVariant::from_int(ITEM_TYPE_FOLDER), ITEM_TYPE);
                                state_item.set_data_2a(&QVariant::from_int(ITEM_STATUS_PRISTINE), ITEM_STATUS);
                                state_item.set_editable(false);
                                state_item.set_flags(QFlags::from(flags));
                                let icon_type = IconType::Folder;
                                icon_type.set_icon_to_item_safe(&folder);

                                let qlist = QListOfQStandardItem::new();
                                qlist.append_q_standard_item(&folder.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&state_item.into_ptr().as_mut_raw_ptr());
                                parent.append_row_q_list_of_q_standard_item(qlist.as_ref());

                                // This is our parent now.
                                let index = parent.row_count() - 1;
                                parent = parent.child_1a(index);
                            }
                        }
                    }
                }

                // Delay adding the big parent as much as we can, as otherwise the signals triggered when adding a PackedFile can slow this down to a crawl.
                let qlist = QListOfQStandardItem::new();
                qlist.append_q_standard_item(&big_parent.into_ptr().as_mut_raw_ptr());
                qlist.append_q_standard_item(&state_item.into_ptr().as_mut_raw_ptr());

                model.append_row_q_list_of_q_standard_item(qlist.as_ref());
                self.header().set_section_resize_mode_2a(0, ResizeMode::Stretch);
                self.header().set_section_resize_mode_2a(1, ResizeMode::Interactive);
                self.header().set_minimum_section_size(4);
                self.header().resize_section(1, 4);
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
                let response = CENTRAL_COMMAND.recv_message_qt();
                let packed_files_info = if let Response::VecOptionPackedFileInfo(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
                for (item_type, packed_file_info) in item_types.iter().zip(packed_files_info.iter()) {

                    // We only use this to add files and empty folders. Ignore the rest.
                    if let TreePathType::File(ref path) | TreePathType::Folder(ref path) = &item_type {
                        let mut parent = model.item_1a(0);
                        let mut parent_status = get_status_item_from_item(parent);
                        let flags = ItemFlag::from(parent_status.flags().to_int() & ItemFlag::ItemIsSelectable.to_int());
                        match parent_status.data_1a(ITEM_STATUS).to_int_0a() {
                             ITEM_STATUS_PRISTINE => parent_status.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED), ITEM_STATUS),
                             ITEM_STATUS_MODIFIED => parent_status.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED | ITEM_STATUS_MODIFIED), ITEM_STATUS),
                             ITEM_STATUS_ADDED | 3 => {},
                             _ => unimplemented!(),
                        }
                        if !parent_status.data_1a(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                            parent_status.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
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
                                        let child = parent.child_2a(index, 0);

                                        // We ignore files or folders, depending on what we want to create.
                                        if let TreePathType::File(_) = &item_type {
                                            if child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FOLDER { continue }
                                        }

                                        if let TreePathType::Folder(_) = &item_type {
                                            if child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { continue }
                                        }

                                        // Get his text. If it's the same file/folder we are trying to add, this is the one.
                                        if child.text().to_std_string() == *name {
                                            parent = parent.child_1a(index);
                                            parent_status = get_status_item_from_item(parent);
                                            duplicate_found = true;
                                            break;
                                        }
                                    }
                                }

                                // If the item already exist, re-use it.
                                if duplicate_found {
                                    parent_status.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED), ITEM_STATUS);
                                    parent_status.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                                }

                                // Otherwise, it's a new PackedFile, so do the usual stuff.
                                else {

                                    // Create the Item, configure it depending on if it's a file or a folder,
                                    // and add the file to the TreeView.
                                    let item = QStandardItem::from_q_string(&QString::from_std_str(name)).into_ptr();
                                    let item_status = QStandardItem::new().into_ptr();
                                    item.set_editable(false);
                                    item_status.set_editable(false);

                                    if let TreePathType::File(ref path) = &item_type {
                                        item.set_data_2a(&QVariant::from_int(ITEM_TYPE_FILE), ITEM_TYPE);
                                        IconType::set_icon_to_item_safe(&IconType::File(path.to_vec()), &item);
                                        if let Some(info) = packed_file_info {
                                            let tooltip = new_packed_file_tooltip(info);
                                            item.set_tool_tip(&QString::from_std_str(tooltip));
                                        }
                                    }

                                    else if let TreePathType::Folder(_) = &item_type {
                                        item.set_data_2a(&QVariant::from_int(ITEM_TYPE_FOLDER), ITEM_TYPE);
                                        IconType::set_icon_to_item_safe(&IconType::Folder, &item);
                                    }

                                    let qlist = QListOfQStandardItem::new().into_ptr();
                                    qlist.append_q_standard_item(&item.as_mut_raw_ptr());
                                    qlist.append_q_standard_item(&item_status.as_mut_raw_ptr());

                                    parent.append_row_q_list_of_q_standard_item(qlist.as_ref().unwrap());

                                    item_status.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                                    item_status.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED), ITEM_STATUS);
                                    item_status.set_flags(QFlags::from(flags));

                                    // Sort the TreeView.
                                    sort_item_in_tree_view(
                                        &model,
                                        item,
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
                                        let child = parent.child_2a(index, 0);
                                        if child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { continue }

                                        // Get his text. If it's the same folder we are trying to add, this is our parent now.
                                        if child.text().to_std_string() == *name {
                                            parent = parent.child_1a(index);
                                            parent_status = get_status_item_from_item(parent);
                                            match parent_status.data_1a(ITEM_STATUS).to_int_0a() {
                                                 ITEM_STATUS_PRISTINE => parent_status.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED), ITEM_STATUS),
                                                 ITEM_STATUS_MODIFIED => parent_status.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED | ITEM_STATUS_MODIFIED), ITEM_STATUS),
                                                 ITEM_STATUS_ADDED | 3 => {},
                                                 _ => unimplemented!(),
                                            }

                                            if !parent_status.data_1a(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                                                parent_status.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                                            }

                                            duplicate_found = true;
                                            break;
                                        }
                                    }
                                }


                                // If the folder doesn't already exists, just add it.
                                if !duplicate_found {
                                    let folder = QStandardItem::from_q_string(&QString::from_std_str(name)).into_ptr();
                                    let folder_status = QStandardItem::new().into_ptr();
                                    folder.set_editable(false);
                                    folder.set_data_2a(&QVariant::from_int(ITEM_TYPE_FOLDER), ITEM_TYPE);

                                    IconType::set_icon_to_item_safe(&IconType::Folder, &folder);

                                    let qlist = QListOfQStandardItem::new();
                                    qlist.append_q_standard_item(&folder.as_mut_raw_ptr());
                                    qlist.append_q_standard_item(&folder_status.as_mut_raw_ptr());

                                    parent.append_row_q_list_of_q_standard_item(qlist.as_ref());

                                    folder_status.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED), ITEM_STATUS);
                                    folder_status.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                                    folder_status.set_editable(false);
                                    folder_status.set_flags(QFlags::from(flags));

                                    // This is our parent now.
                                    let index = parent.row_count() - 1;
                                    parent = parent.child_1a(index);
                                    parent_status = get_status_item_from_item(parent);

                                    // Sort the TreeView.
                                    sort_item_in_tree_view(
                                        &model,
                                        folder,
                                        &TreePathType::Folder(vec![String::new()])
                                    );
                                }
                            }
                        }

                        if SETTINGS.read().unwrap().settings_bool["expand_treeview_when_adding_items"] {
                            self.expand_treeview_to_item(path);
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
                            let packfile = model.item_1a(0);
                            let mut item = model.item_1a(0);
                            let mut index = 0;
                            let path_deep = path.len();

                            // First looping downwards.
                            loop {

                                // If we reached the folder of the file, search through all his children for the file we want.
                                if index == (path_deep - 1) {
                                    for row in 0..item.row_count() {
                                        let child = item.child_1a(row);
                                        if child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FOLDER { continue }

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
                                        let child = item.child_1a(row);
                                        if child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { continue }

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
                            let mut parent_status;

                            // Begin the endless cycle of war and dead.
                            loop {

                                // Get the parent of the item, and kill the item in a cruel way.
                                parent = item.parent();
                                parent_status = get_status_item_from_item(parent);

                                // Block the selection from retriggering open PackedFiles.
                                let blocker = QSignalBlocker::from_q_object(&self.selection_model());
                                parent.remove_row(item.row());
                                blocker.unblock();

                                parent_status.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS);
                                if !parent_status.data_1a(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                                    parent_status.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                                }

                                // If the parent has more children, or we reached the PackFile, we're done. Otherwise, we update our item.
                                if parent.has_children() || !packfile.has_children() { break; }
                                else { item = parent }
                            }

                            // Third time's a charm.
                            if let TreePathType::Folder(ref path) = Self::get_type_from_item(parent, &model) {
                                for _ in 0..path.len() {
                                    parent_status.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS);
                                    parent_status.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                                    parent = parent.parent();
                                    parent_status = get_status_item_from_item(parent);
                                }
                            }
                        }

                        TreePathType::Folder(path) => {

                            // Get the PackFile's item and the one we're gonna swap around, and the info to see how deep must we go.
                            let packfile = model.item_1a(0);
                            let mut item = model.item_1a(0);
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
                                        let child = item.child_1a(row);
                                        if child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { continue }

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
                            let mut parent_status;

                            // Begin the endless cycle of war and dead.
                            loop {

                                // Get the parent of the item and kill the item in a cruel way.
                                parent = item.parent();
                                parent_status = get_status_item_from_item(parent);

                                let blocker = QSignalBlocker::from_q_object(&self.selection_model());
                                parent.remove_row(item.row());
                                blocker.unblock();

                                parent_status.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS);
                                if !parent_status.data_1a(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                                    parent_status.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                                }

                                // If the parent has more children, or we reached the PackFile, we're done. Otherwise, we update our item.
                                if parent.has_children() | !packfile.has_children() { break; }
                                else { item = parent }
                            }

                            // Third time's a charm.
                            if let TreePathType::Folder(ref path) = Self::get_type_from_item(parent, &model) {
                                for _ in 0..path.len() {
                                    parent_status.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS);
                                    parent_status.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                                    parent = parent.parent();
                                    parent_status = get_status_item_from_item(parent);
                                }
                            }
                        }

                        TreePathType::PackFile => self.update_treeview(true, TreeViewOperation::Build(None, None)),

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

                            let item = Self::get_item_from_type(&path_type, &model);
                            let item = get_status_item_from_item(item);
                            match item.data_1a(ITEM_STATUS).to_int_0a() {
                                ITEM_STATUS_PRISTINE => item.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS),
                                ITEM_STATUS_ADDED => item.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED | ITEM_STATUS_MODIFIED), ITEM_STATUS),
                                ITEM_STATUS_MODIFIED | 3 => {},
                                _ => unimplemented!(),
                            };

                            // If its a file, we get his new info and put it in a tooltip.
                            if let TreePathType::File(_) = path_type {
                                CENTRAL_COMMAND.send_message_qt(Command::GetPackedFileInfo(path.to_vec()));
                                let response = CENTRAL_COMMAND.recv_message_qt();
                                let packed_file_info = if let Response::OptionPackedFileInfo(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
                                if let Some(info) = packed_file_info {
                                    let tooltip = new_packed_file_tooltip(&info);
                                    item.set_tool_tip(&QString::from_std_str(tooltip));
                                }
                            }

                            let cycles = if !path.is_empty() { path.len() } else { 0 };
                            let mut parent = get_status_item_from_item(item.parent());
                            for _ in 0..cycles {

                                // Get the status and mark them as needed.
                                match parent.data_1a(ITEM_STATUS).to_int_0a() {
                                    ITEM_STATUS_PRISTINE => parent.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS),
                                    ITEM_STATUS_ADDED => parent.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED | ITEM_STATUS_MODIFIED), ITEM_STATUS),
                                    ITEM_STATUS_MODIFIED | 3 => {},
                                    _ => unimplemented!(),
                                };

                                // Set the new parent.
                                parent = get_status_item_from_item(parent.parent());
                            }
                        }

                        TreePathType::PackFile => {
                            let item = model.item_2a(0, 1);
                            if !item.is_null() {
                                let status = item.data_1a(ITEM_STATUS).to_int_0a();
                                match status {
                                    ITEM_STATUS_PRISTINE => item.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS),
                                    ITEM_STATUS_ADDED => item.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED | ITEM_STATUS_MODIFIED), ITEM_STATUS),
                                    ITEM_STATUS_MODIFIED | 3 => {},
                                    _ => unimplemented!(),
                                };
                                item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                            }
                        }

                        TreePathType::None => return,
                    }
                }
            }

            // If we want to move something. The logic here is, first we remove the item from its current position,
            // then we add it to the new one, keeping its attributes.
            TreeViewOperation::Move(path_types) => {

                // First, get the `PackedFileInfo` of each of the new paths (so we can later build their tooltip, if neccesary).
                let new_paths = path_types.iter().map(|(_, y)| y.to_vec()).collect::<Vec<Vec<String>>>();
                CENTRAL_COMMAND.send_message_qt(Command::GetPackedFilesInfo(new_paths));
                let response = CENTRAL_COMMAND.recv_message_qt();
                let packed_files_info = if let Response::VecOptionPackedFileInfo(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };

                for ((path_type, new_path), packed_file_info) in path_types.iter().zip(packed_files_info.iter())  {
                    let taken_row = Self::take_row_from_type(path_type, &model);
                    Self::add_row_to_path(taken_row, &model, new_path, packed_file_info);
                    self.expand_treeview_to_item(&new_path);
                }
            },

            // If you want to mark an item so it can't lose his modified state...
            TreeViewOperation::MarkAlwaysModified(item_types) => {
                for item_type in &item_types {
                    let item = Self::get_item_from_type(item_type, &model);
                    let item = get_status_item_from_item(item);
                    if !item.is_null() && !item.data_1a(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                        item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                    }
                }
            }

            // If we want to undo the doings of any PackFile.
            TreeViewOperation::Undo(item_types) => {
                for item_type in item_types {
                    match item_type {
                        TreePathType::File(ref path) | TreePathType::Folder(ref path) => {

                            // Get the item and only try to restore it if we didn't set it as "not to restore".
                            let item = Self::get_item_from_type(&item_type, &model);
                            let item = get_status_item_from_item(item);
                            if !item.data_1a(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                                if item.data_1a(ITEM_STATUS).to_int_0a() != ITEM_STATUS_PRISTINE {
                                    item.set_data_2a(&QVariant::from_int(ITEM_STATUS_PRISTINE), ITEM_STATUS);
                                }

                                // If its a file, we get his new info and put it in a tooltip.
                                if let TreePathType::File(_) = item_type {
                                    CENTRAL_COMMAND.send_message_qt(Command::GetPackedFileInfo(path.to_vec()));
                                    let response = CENTRAL_COMMAND.recv_message_qt();
                                    let packed_file_info = if let Response::OptionPackedFileInfo(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
                                    if let Some(info) = packed_file_info {
                                        let tooltip = new_packed_file_tooltip(&info);
                                        item.set_tool_tip(&QString::from_std_str(tooltip));
                                    }
                                }

                                // Get the times we must to go up until we reach the parent.
                                let cycles = if !path.is_empty() { path.len() } else { 0 };
                                let mut parent = get_status_item_from_item(item.parent());

                                // Unleash hell upon the land.
                                for _ in 0..cycles {

                                    if !parent.data_1a(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                                        if parent.data_1a(ITEM_STATUS).to_int_0a() != ITEM_STATUS_PRISTINE {
                                            parent.set_data_2a(&QVariant::from_int(ITEM_STATUS_PRISTINE), ITEM_STATUS);
                                        }
                                        else { break; }
                                    }
                                    else { break; }
                                    parent = get_status_item_from_item(parent.parent());
                                }
                            }
                        }

                        // This one is a bit special. We need to check, not only him, but all his children too.
                        TreePathType::PackFile => {
                            let item = model.item_2a(0, 1);
                            let mut packfile_is_modified = false;
                            for row in 0..item.row_count() {
                                let child = item.child_2a(row, 1);
                                if child.data_1a(ITEM_STATUS).to_int_0a() != ITEM_STATUS_PRISTINE {
                                    packfile_is_modified = true;
                                    break;
                                }
                            }

                            if !packfile_is_modified {
                                item.set_data_2a(&QVariant::from_int(ITEM_STATUS_PRISTINE), ITEM_STATUS);
                            }
                        }
                        TreePathType::None => unimplemented!(),
                    }
                }
            }

            // If we want to remove the colour of the TreeView...
            TreeViewOperation::Clean => clean_treeview(None, &model),

            // If we want to remove everything from the TreeView...
            TreeViewOperation::Clear => model.clear(),

            // If we want to get the tooltips of the PackedFiles updated...
            TreeViewOperation::UpdateTooltip(packed_files_info) => {
                for packed_file_info in packed_files_info {
                    let tooltip = QString::from_std_str(&new_packed_file_tooltip(&packed_file_info));
                    let tree_path_type = TreePathType::File(packed_file_info.path.to_vec());
                    let item = Self::get_item_from_type(&tree_path_type, &model);
                    item.set_tool_tip(&tooltip);
                }
            },
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
unsafe fn clean_treeview(item: Option<Ptr<QStandardItem>>, model: &QStandardItemModel) {

    // If we receive None, use the PackFile.
    if model.row_count_0a() > 0 {
        let item = if let Some(item) = item { item } else { model.item_2a(0, 1) };

        // Clean the current item, and repeat for each children.
        item.set_data_2a(&QVariant::from_int(ITEM_STATUS_PRISTINE), ITEM_STATUS);
        item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_FOREVER_MODIFIED);
        let children_count = item.row_count();
        for row in 0..children_count {
            let child = item.child_2a(row, 1);
            clean_treeview(Some(child), model);
        }
    }
}

/// This function returns the currently visible childs of the given parent, and add them as `TreePathType`s to the provided list.
unsafe fn get_visible_childs_of_item(parent: &QStandardItem, tree_view: &QTreeView, filter: &QSortFilterProxyModel, model: &QPtr<QStandardItemModel>, item_types: &mut Vec<TreePathType>) {
    for row in 0..parent.row_count() {
        let child = parent.child_1a(row);
        let child_index = child.index();
        let filtered_index = filter.map_from_source(&child_index);
        if filtered_index.is_valid() {
            if child.has_children() {
                get_visible_childs_of_item(&child, tree_view, filter, model, item_types);
            }
            else {
                item_types.push(<QBox<QTreeView> as PackTree>::get_type_from_item(child, model));
            }
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
unsafe fn sort_item_in_tree_view(
    model: &QPtr<QStandardItemModel>,
    mut item: Ptr<QStandardItem>,
    item_type: &TreePathType,
) {

    // Get the ModelIndex of our Item and his row, as that's what we are going to be changing.
    let mut item_index = item.index();

    // Get the parent of the item.
    let parent = item.parent();
    let parent_index = parent.index();

    // Get the previous and next item ModelIndex on the list.
    let item_index_prev = model.index_3a(item_index.row() - 1, item_index.column(), &parent_index);
    let item_index_next = model.index_3a(item_index.row() + 1, item_index.column(), &parent_index);

    // Get the type of the previous item on the list.
    let item_type_prev: TreePathType = if item_index_prev.is_valid() {
        let item_sibling = model.item_from_index(&item_index_prev);
        <QBox<QTreeView>>::get_type_from_item(item_sibling, &model)
    }

    // Otherwise, return the type as `None`.
    else { TreePathType::None };

    // Get the type of the previous and next items on the list.
    let item_type_next: TreePathType = if item_index_next.is_valid() {

        // Get the next item.
        let item_sibling = model.item_from_index(&item_index_next);
        <QBox<QTreeView>>::get_type_from_item(item_sibling, &model)
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
        *item_type == TreePathType::Folder(vec![String::new()])
    }

    // If the two around it are the same type, compare them and decide.
    else {

        // Get the previous, current and next texts.
        let previous_name = parent.child_1a(item_index.row() - 1).text().to_std_string();
        let current_name = parent.child_1a(item_index.row()).text().to_std_string();
        let next_name = parent.child_1a(item_index.row() + 1).text().to_std_string();

        // If, after sorting, the previous hasn't changed position, it shouldn't go up.
        let name_list = vec![previous_name.to_owned(), current_name.to_owned()];
        let mut name_list_sorted = vec![previous_name, current_name.to_owned()];
        name_list_sorted.sort();
        if name_list == name_list_sorted {

            // If, after sorting, the next hasn't changed position, it shouldn't go down.
            let name_list = vec![current_name.to_owned(), next_name.to_owned()];
            let mut name_list_sorted = vec![current_name, next_name];
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
            let item_sibling = parent.child_1a(item_sibling_index.row());
            let item_sibling_type = <QBox<QTreeView>>::get_type_from_item(item_sibling, &model);

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
                        parent.insert_row_int_q_list_of_q_standard_item(item_sibling_index.row(), &item_x);
                        item = parent.child_1a(item_sibling_index.row());
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
                        parent.insert_row_int_q_list_of_q_standard_item(item_sibling_index.row(), &item_x);
                        item = parent.child_1a(item_sibling_index.row());
                        item_index = item.index();
                    }
                }
            }

            // If the top one is a File and the bottom one a Folder, it's an special situation. Just swap them.
            else if *item_type == TreePathType::Folder(vec![String::new()]) && item_sibling_type == TreePathType::File(vec![String::new()]) {

                // We swap them, and update them for the next loop.
                let item_x = parent.take_row(item_index.row());
                parent.insert_row_int_q_list_of_q_standard_item(item_sibling_index.row(), &item_x);
                item = parent.child_1a(item_sibling_index.row());
                item_index = item.index();
            }

            // If the type is different and it's not an special situation, we can't move anymore.
            else { break; }
        }

        // If the Item is invalid, we can't move anymore.
        else { break; }
    }
}

pub unsafe fn get_color_added() -> Ptr<QColor> {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        QColor::from_q_string(&QString::from_std_str(*GREEN_DARK)).into_ptr()
    } else {
        QColor::from_q_string(&QString::from_std_str(*GREEN_BRIGHT)).into_ptr()
    }
}

pub unsafe fn get_color_modified() -> Ptr<QColor> {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        QColor::from_q_string(&QString::from_std_str(*YELLOW_DARK)).into_ptr()
    } else {
        QColor::from_q_string(&QString::from_std_str(*YELLOW_BRIGHT)).into_ptr()
    }
}

pub unsafe fn get_color_added_modified() -> Ptr<QColor> {
    let color_added = get_color_added();
    let color_modified = get_color_modified();
    let color = QColor::new();
    color.set_red((color_added.red() + color_modified.red()) / 2);
    color.set_green((color_added.green() + color_modified.green()) / 2);
    color.set_blue((color_added.blue() + color_modified.blue()) / 2);
    color.set_alpha((color_added.alpha() + color_modified.alpha()) / 2);
    color.into_ptr()
}

pub unsafe fn get_color_unmodified() -> Ptr<QColor> {
    QColor::from_global_color(GlobalColor::Transparent).into_ptr()
}

pub unsafe fn get_color_correct() -> String {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        GREEN_DARK.to_owned()
    } else {
        GREEN_BRIGHT.to_owned()
    }
}

pub unsafe fn get_color_wrong() -> String {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        RED_DARK.to_owned()
    } else {
        RED_BRIGHT.to_owned()
    }
}

pub unsafe fn get_color_clean() -> String {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        MEDIUM_DARKER_GREY.to_owned()
    } else {
        TRANSPARENT_BRIGHT.to_owned()
    }
}

pub unsafe fn get_color_info() -> String {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        INFO_UNPRESSED_DARK.to_owned()
    } else {
        INFO_UNPRESSED_LIGHT.to_owned()
    }
}

pub unsafe fn get_color_warning() -> String {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        WARNING_UNPRESSED_DARK.to_owned()
    } else {
        WARNING_UNPRESSED_LIGHT.to_owned()
    }
}

pub unsafe fn get_color_warning_foreground() -> String {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        WARNING_UNPRESSED_DARK.to_owned()
    } else {
        WARNING_FOREGROUND_LIGHT.to_owned()
    }
}

pub unsafe fn get_color_error() -> String {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        ERROR_UNPRESSED_DARK.to_owned()
    } else {
        ERROR_UNPRESSED_LIGHT.to_owned()
    }
}

pub unsafe fn get_color_error_foreground() -> String {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        ERROR_UNPRESSED_DARK.to_owned()
    } else {
        ERROR_FOREGROUND_LIGHT.to_owned()
    }
}

pub unsafe fn get_color_info_pressed() -> String {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        INFO_PRESSED_DARK.to_owned()
    } else {
        INFO_PRESSED_LIGHT.to_owned()
    }
}

pub unsafe fn get_color_warning_pressed() -> String {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        WARNING_PRESSED_DARK.to_owned()
    } else {
        WARNING_PRESSED_LIGHT.to_owned()
    }
}

pub unsafe fn get_color_error_pressed() -> String {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        ERROR_PRESSED_DARK.to_owned()
    } else {
        ERROR_PRESSED_LIGHT.to_owned()
    }
}

pub unsafe fn get_color_added_high_intensity() -> Ptr<QColor> {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        QColor::from_q_string(&QString::from_std_str(*GREEN_DARK)).into_ptr()
    } else {
        QColor::from_q_string(&QString::from_std_str(*GREEN_MEDIUM)).into_ptr()
    }
}

pub unsafe fn get_color_modified_high_intensity() -> Ptr<QColor> {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        QColor::from_q_string(&QString::from_std_str(*YELLOW_DARK)).into_ptr()
    } else {
        QColor::from_q_string(&QString::from_std_str(*YELLOW_MEDIUM)).into_ptr()
    }
}

pub unsafe fn get_color_added_modified_high_intensity() -> Ptr<QColor> {
    QColor::from_q_string(&QString::from_std_str(*MAGENTA_MEDIUM)).into_ptr()
}


/*pub unsafe fn get_color_deleted() -> Ptr<QColor> {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        QColor::from_q_string(&QString::from_std_str(*RED_DARK)).into_ptr()
    } else {
        QColor::from_q_string(&QString::from_std_str(*RED_BRIGHT)).into_ptr()
    }
}*/

unsafe fn get_status_item_from_item(item: Ptr<QStandardItem>) -> Ptr<QStandardItem> {
    if !item.is_null() {
        if item.parent().is_null() {
            item.model().item_2a(0, 1)
        } else {
            item.parent().child_2a(item.row(), 1)
        }
    }

    // If the item is null, we fuck it up somewhere else.
    else {
        item
    }
}
