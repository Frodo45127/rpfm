//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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

use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;
use qt_gui::QListOfQStandardItem;

use qt_core::QModelIndex;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QVariant;
use qt_core::QPtr;

use cpp_core::CppBox;
use cpp_core::Ptr;
use cpp_core::Ref;
use cpp_core::CastFrom;

use rayon::prelude::*;
use time::OffsetDateTime;

use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use rpfm_lib::files::{ContainerPath, FileType, pack::PFHFlags};
use rpfm_lib::integrations::log::error;
use rpfm_lib::utils::*;

use rpfm_ui_common::FULL_DATE_FORMAT;
use rpfm_ui_common::locale::qtr;

use crate::backend::*;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::packedfile_views::DataSource;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::settings_ui::backend::*;
use crate::TREEVIEW_ICONS;
use crate::utils::*;

// This one is needed for initialization on boot, so it has to be public.
pub mod icons;

/// This const is the key of the QVariant that holds the type of each StandardItem in a `TreeView`.
const ITEM_TYPE: i32 = 20;

/// This const is the key of the QVariant that holds the status of each StandardItem in a `TreeView`.
const ITEM_STATUS: i32 = 21;

/// This const is the key of the QVariant that holds if the item changed state should be *undoable* or not.
const ITEM_IS_FOREVER_MODIFIED: i32 = 22;

/// This const is the key of the QVariant that holds what kind of Root Node we have. Only in root nodes.
const ROOT_NODE_TYPE: i32 = 23;

/// This const is used to identify an editable PackFile.
const ROOT_NODE_TYPE_EDITABLE_PACKFILE: i32 = 0;

/// This const is used to identify a non-editable PackFile.
const ROOT_NODE_TYPE_NON_EDITABLE_PACKFILE: i32 = 1;

/// This const is used to identify an Asskit node.
const ROOT_NODE_TYPE_ASSKIT: i32 = 2;

/// This const is used to identify a Game data node.
const ROOT_NODE_TYPE_GAME_DATA: i32 = 3;

/// This const is used to identify a Parent data node.
const ROOT_NODE_TYPE_PARENT_DATA: i32 = 4;

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

//-------------------------------------------------------------------------------//
//                          Enums & Structs (and trait)
//-------------------------------------------------------------------------------//

/// This trait adds multiple util functions to the `TreeView` you implement it for.
///
/// Keep in mind that this trait has been created with `PackFile TreeView's` in mind, so his methods
/// may not be suitable for all purposes.
pub trait PackTree {

    /// This function allows us to add the provided item into the path we want on the `TreeView`, taking care of adding missing parents.
    unsafe fn add_row_to_path(item: Ptr<QListOfQStandardItem>, model: &QPtr<QStandardItemModel>, path: &str, file_info: Option<&RFileInfo>);

    /// This function is used to expand the entire path from the PackFile to an specific item in the `TreeView`.
    ///
    /// It returns the `ModelIndex` of the final item of the path, or None if it wasn't found or it's hidden by the filter.
    unsafe fn expand_treeview_to_item(&self, path: &str, source: DataSource) -> Option<Ptr<QModelIndex>>;

    /// This function is used to expand an item and all it's children recursively.
    unsafe fn expand_all_from_item(tree_view: &QTreeView, item: Ptr<QStandardItem>, first_item: bool);

    /// This function is used to expand an item and all it's children recursively.
    unsafe fn expand_all_from_type(tree_view: &QTreeView, item: &ContainerPath);

    /// This function gives you the items selected in the PackFile Content's TreeView.
    unsafe fn get_items_from_main_treeview_selection(pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Vec<Ptr<QStandardItem>>;

    /// This function gives you the items selected in the provided `TreeView`.
    unsafe fn get_items_from_selection(&self, has_filter: bool) -> Vec<Ptr<QStandardItem>>;

    /// This function gives you the `TreeViewTypes` of the items selected in the PackFile Content's TreeView.
    unsafe fn get_item_types_from_main_treeview_selection(pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Vec<ContainerPath>;

    /// This function gives you the `TreeViewTypes` of the items selected in the provided TreeView.
    unsafe fn get_item_types_from_selection(&self, has_filter: bool) -> Vec<ContainerPath>;

    /// This function returns the `ContainerPath`s not hidden by the applied filter corresponding to the current selection.
    ///
    /// This always assumes the `TreeView` has a filter. It'll die horrendously otherwise.
    unsafe fn get_item_types_from_selection_filtered(&self) -> Vec<ContainerPath>;

    /// This function gives you the `ContainerPaths` and source of all items in a TreeView.
    unsafe fn get_item_types_and_data_source_from_selection(&self, has_filter: bool) -> Vec<(ContainerPath, DataSource)>;

    /// This function gives you the item corresponding to an specific `ContainerPath`.
    unsafe fn item_from_path(path: &ContainerPath, model: &QPtr<QStandardItemModel>) -> Ptr<QStandardItem>;

    /// This function gives you the DataSource of the selection of the provided TreeView.
    unsafe fn get_root_source_type_from_selection(&self, has_filter: bool) -> Option<DataSource>;

    /// This function gives you the DataSource of the index of the provided TreeView.
    unsafe fn get_root_source_type_from_index(&self, index: CppBox<QModelIndex>) -> DataSource;

    /// This function gives you a bitmask with what's selected in the PackFile Content's TreeView,
    /// the number of selected files, and the number of selected folders.
    unsafe fn get_combination_from_main_treeview_selection(pack_file_contents_ui: &Rc<PackFileContentsUI>) -> (u8, u32, u32, u32);

    /// This function returns the `ContainerPath` of the provided item. Unsafe version.
    unsafe fn get_type_from_item(item: Ptr<QStandardItem>, model: &QPtr<QStandardItemModel>) -> ContainerPath;

    /// This function is used to get the path of a specific Item in a StandardItemModel. Unsafe version.
    unsafe fn get_path_from_item(item: Ptr<QStandardItem>, model: &QPtr<QStandardItemModel>) -> String;

    /// This function is used to get the path of a specific ModelIndex in a StandardItemModel. Unsafe version.
    unsafe fn get_path_from_index(index: Ref<QModelIndex>, model: &QPtr<QStandardItemModel>) -> String;

    /// This function gives you the path of the items selected in the provided TreeView.
    unsafe fn get_path_from_selection(&self) -> Vec<String>;

    /// This function gives you the path it'll have in the PackFile Content's TreeView a file from disk.
    unsafe fn get_path_from_pathbuf(pack_file_contents_ui: &Rc<PackFileContentsUI>, file_path: &Path, is_file: bool) -> Vec<String>;

    /// This function removes the item under the provided path and returns it, removing it from the tree.
    unsafe fn take_row_from_path(path: &ContainerPath, model: &QPtr<QStandardItemModel>) -> Ptr<QListOfQStandardItem>;

    /// This function returns the currently visible children of the given parent, and adds them as `ContainerPath`s to the provided list.
    unsafe fn visible_children_of_item(&self, parent: &QStandardItem, visible_paths: &mut Vec<ContainerPath>);

    /// This function takes care of EVERY operation that manipulates the provided TreeView.
    /// It does one thing or another, depending on the operation we provide it.
    ///
    /// BIG NOTE: Each StandardItem should keep track of his own status, meaning that their data means:
    /// - Position 20: Type. 1 is File, 2 is Folder, 4 is PackFile.
    /// - Position 21: Status. 0 is untouched, 1 is added, 2 is modified.
    /// In case you don't realise, those are bitmasks.
    unsafe fn update_treeview(&self, has_filter: bool, operation: TreeViewOperation, source: DataSource);
}

/// This enum has the different possible operations we can do in a `TreeView`.
#[derive(Clone, Debug)]
pub enum TreeViewOperation {

    /// Build the entire `TreeView` from nothing. Requires an option: Some<PathBuf> if the `PackFile` is not editable, `None` if it is.
    /// Also, you can pass a ContainerInfo/RFileInfo if you want to build a TreeView with custom data.
    Build(BuildData),

    /// Add one or more files/folders to the `TreeView`. Requires a `Vec<ContainerPath>` to add to the `TreeView`.
    Add(Vec<ContainerPath>),

    /// Remove the files/folders corresponding to the `Vec<ContainerPath>` we provide from the `TreeView`.
    Delete(Vec<ContainerPath>),

    /// Set the provided paths as *modified*. It requires the `Vec<ContainerPath>` of whatever you want to mark as modified.
    Modify(Vec<ContainerPath>),

    /// Change the name of a file/folder from the `TreeView`. Requires the `ContainerPath` of whatever you want to move,
    /// its new path and the base folders that must be removed with the move, if any.
    Move(Vec<(ContainerPath, ContainerPath)>, Vec<ContainerPath>),

    /// Mark an item as ***Always Modified*** so it cannot be marked as unmodified by an undo operation.
    MarkAlwaysModified(Vec<ContainerPath>),

    /// Resets the state of one or more `ContainerPath` to 0, or unmodified.
    Undo(Vec<ContainerPath>),

    /// Remove all status and color from the entire `TreeView`.
    Clean,

    /// Remove all items from the `TreeView`.
    Clear,

    /// Updates the tooltip of the PackedFiles with the provided info.
    UpdateTooltip(Vec<RFileInfo>),
}

/// This struct represents the data needed to build a TreeView.
#[derive(Clone, Debug)]
pub struct BuildData {

    /// The path on disk of the PackFile we're trying to open, in case it has one.
    pub path: Option<PathBuf>,

    /// The "data" to load this instead of a PackFile from the backend.
    pub data: Option<(ContainerInfo, Vec<RFileInfo>)>,

    /// If this Tree is editable or not (for the root icon).
    pub editable: bool,
}

//-------------------------------------------------------------------------------//
//                      Implementations of `PackTree`
//-------------------------------------------------------------------------------//

impl PackTree for QPtr<QTreeView> {

    unsafe fn add_row_to_path(row: Ptr<QListOfQStandardItem>, model: &QPtr<QStandardItemModel>, path: &str, file_info: Option<&RFileInfo>) {

        // First, mark the root item as forever modified.
        let mut item = model.item_1a(0);
        item.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS);
        item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);

        // Then looping downwards. -1 because we want to reach the "parent" that will hold the new row, not the row itself.
        let path = path.split('/').collect::<Vec<_>>();
        for path_item in &path[..path.len() - 1] {
            let name_q_string = QString::from_std_str(path_item);
            let mut item_found = false;

            for row in 0..item.row_count() {
                let child = item.child_1a(row);

                // We are only interested in folders.
                if child.data_1a(ITEM_TYPE).to_int_0a() != ITEM_TYPE_FOLDER { continue }

                // If we found it, we're done.
                if child.text().compare_q_string(&name_q_string) == 0 {
                    child.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS);
                    child.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                    item = child;
                    item_found = true;
                    break;
                }
            }

            // If we haven't found the item in question, we need to create it and set it as our new item.
            if !item_found {
                let folder_item = QStandardItem::from_q_string(&name_q_string);
                folder_item.set_editable(false);
                folder_item.set_data_2a(&QVariant::from_int(ITEM_TYPE_FOLDER), ITEM_TYPE);
                folder_item.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS);
                folder_item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                TREEVIEW_ICONS.set_standard_item_icon(&folder_item, None);

                item.append_row_q_standard_item(folder_item.into_ptr());

                let index = item.row_count() - 1;
                item = item.child_1a(index);

                sort_item_in_tree_view(model, item, &ContainerPath::Folder(String::new()));
            }
        }

        // If we have fileinfo, set the new tooltip for the item.
        if let Some(file_info) = file_info {
            let tooltip = new_packed_file_tooltip(file_info);
            if !tooltip.is_empty() {
                row.value_1a(0).set_tool_tip(&QString::from_std_str(tooltip));
            }
        }

        // If there was an item with than name, remove it.
        let type_to_skip = if row.value_1a(0).data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { ITEM_TYPE_FOLDER } else { ITEM_TYPE_FILE };
        let name_q_string = QString::from_std_str(path.last().unwrap());
        for row in 0..item.row_count() {
            let child = item.child_1a(row);

            if child.data_1a(ITEM_TYPE).to_int_0a() == type_to_skip { continue }

            if child.text().compare_q_string(&name_q_string) == 0 {
                item.remove_row(row);
                break;
            }
        }

        // Then, mark the item as modified, change its name with the new one, and add it to the model.
        row.value_1a(0).set_text(&QString::from_std_str(path.last().unwrap()));
        row.value_1a(0).set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS);
        row.value_1a(0).set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);

        item.append_row_q_list_of_q_standard_item(row.as_ref().unwrap());

        // Sort the TreeView by its proper type.
        let container_path = if type_to_skip == ITEM_TYPE_FILE {
            ContainerPath::Folder(String::new())
        } else {
            ContainerPath::File(String::new())
        };

        sort_item_in_tree_view(model, row.value_1a(0), &container_path);
    }

    unsafe fn expand_treeview_to_item(&self, path: &str, source: DataSource) -> Option<Ptr<QModelIndex>> {
        let filter: QPtr<QSortFilterProxyModel> = self.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();

        //TODO: This needs heavy optimization.

        // Get the first item's index, as that one should always exist (the Packfile).
        let mut item = match source {
            DataSource::PackFile => model.item_1a(0),
            DataSource::ParentFiles => {
                let mut root_item = None;
                for row in 0..model.row_count_0a() {
                    let item = model.item_1a(row);
                    if item.data_1a(ROOT_NODE_TYPE).to_int_0a() == ROOT_NODE_TYPE_PARENT_DATA {
                        root_item = Some(item);
                        break;
                    }
                }

                root_item?
            },
            DataSource::GameFiles => {
                let mut root_item = None;
                for row in 0..model.row_count_0a() {
                    let item = model.item_1a(row);
                    if item.data_1a(ROOT_NODE_TYPE).to_int_0a() == ROOT_NODE_TYPE_GAME_DATA {
                        root_item = Some(item);
                        break;
                    }
                }

                root_item?
            },
            DataSource::AssKitFiles => {
                let mut root_item = None;
                for row in 0..model.row_count_0a() {
                    let item = model.item_1a(row);
                    if item.data_1a(ROOT_NODE_TYPE).to_int_0a() == ROOT_NODE_TYPE_ASSKIT {
                        root_item = Some(item);
                        break;
                    }
                }

                root_item?
            },
            DataSource::ExternalFile => return None,
        };
        let model_index = model.index_2a(0, 0);
        let filtered_index = filter.map_from_source(&model_index);

        // If it's valid (filter didn't hid it away), we expand it and search among its children the next one to expand.
        if filtered_index.is_valid() {
            if !self.is_expanded(&filtered_index) {
                self.expand(&filtered_index);
            }

            // Indexes to see how deep we must go.
            let mut index = 0;
            let path = path.split('/').collect::<Vec<_>>();
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

    unsafe fn expand_all_from_type(tree_view: &QTreeView, item: &ContainerPath) {
        let filter: QPtr<QSortFilterProxyModel> = tree_view.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
        let item = Self::item_from_path(item, &model);
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
        let tree_view = &pack_file_contents_ui.packfile_contents_tree_view();
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

    unsafe fn get_item_types_from_main_treeview_selection(pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Vec<ContainerPath> {
        let items = Self::get_items_from_main_treeview_selection(pack_file_contents_ui);
        items.iter().map(|x| Self::get_type_from_item(*x, &pack_file_contents_ui.packfile_contents_tree_model().static_upcast())).collect()
    }

    unsafe fn get_item_types_from_selection(&self, has_filter: bool) -> Vec<ContainerPath> {
        let items = self.get_items_from_selection(has_filter);

        let model: QPtr<QStandardItemModel> = if has_filter {
            let filter: QPtr<QSortFilterProxyModel> = self.model().static_downcast();
            filter.source_model().static_downcast()
        } else {
            self.model().static_downcast()
        };

        items.iter().map(|x| Self::get_type_from_item(*x, &model)).collect()
    }

    unsafe fn get_item_types_from_selection_filtered(&self)-> Vec<ContainerPath> {
        let filter: QPtr<QSortFilterProxyModel> = self.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();

        let mut item_types = vec![];
        let item_types_selected = self.get_item_types_from_selection(true);
        for item_type in &item_types_selected {
            match item_type {
                 ContainerPath::File(_) => item_types.push(item_type.clone()),
                 ContainerPath::Folder(_) => {
                    let item = Self::item_from_path(item_type, &model);
                    self.visible_children_of_item(&item, &mut item_types);
                 }
            }
        }

        item_types
    }

    unsafe fn visible_children_of_item(&self, parent: &QStandardItem, visible_paths: &mut Vec<ContainerPath>) {
        let filter: QPtr<QSortFilterProxyModel> = self.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();

        for row in 0..parent.row_count() {
            let child = parent.child_1a(row);
            let child_index = child.index();
            let filtered_index = filter.map_from_source(&child_index);
            if filtered_index.is_valid() {
                if child.has_children() {
                    self.visible_children_of_item(&child, visible_paths);
                }
                else {
                    visible_paths.push(Self::get_type_from_item(child, &model));
                }
            }
        }
    }

    unsafe fn get_item_types_and_data_source_from_selection(&self, has_filter: bool) -> Vec<(ContainerPath, DataSource)> {
        let items = self.get_items_from_selection(has_filter);

        let model: QPtr<QStandardItemModel> = if has_filter {
            let filter: QPtr<QSortFilterProxyModel> = self.model().static_downcast();
            filter.source_model().static_downcast()
        } else {
            self.model().static_downcast()
        };

        items.iter().map(|x| (Self::get_type_from_item(*x, &model), self.get_root_source_type_from_index(model.index_from_item(*x)))).collect()
    }

    unsafe fn item_from_path(path: &ContainerPath, model: &QPtr<QStandardItemModel>) -> Ptr<QStandardItem> {
        let mut item = model.item_1a(0);
        let is_file = path.is_file();
        let path = path.path_raw();
        let count = path.split('/').count() - 1;

        for (index, path_element) in path.split('/').enumerate() {
            let children_count = item.row_count();

            // If we reached the folder of the item...
            if index == count {
                let path_element_q_string = QString::from_std_str(path_element);
                for row in 0..children_count {
                    let child = item.child_1a(row);

                    // We ignore files or folders, depending on what we want to create.
                    if is_file && child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FOLDER { continue }
                    if !is_file && child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { continue }

                    let compare = child.text().compare_q_string(&path_element_q_string);
                    match compare.cmp(&0) {
                        Ordering::Equal => {
                            item = child;
                            break;
                        },

                        // If it's less, we still can find the item.
                        Ordering::Less => {}

                        // If it's greater, we passed the item. In theory, this can't happen.
                        Ordering::Greater => {
                            dbg!(child.text().to_std_string());
                            dbg!(path_element_q_string.to_std_string());
                            dbg!("bug?");
                            break;
                        },
                    }
                }
                break;
            }

            // If we are not still in the folder of the file...
            else {

                // Get the amount of children of the current item and go through them until we find our folder.
                let mut not_found = true;
                let text_to_search = QString::from_std_str(path_element);
                for row in 0..children_count {
                    let child = item.child_1a(row);

                    // Items are sorted with folders first. If we start finding files, we already skipped our item.
                    if child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { break; }

                    let compare = QString::compare_2_q_string(child.text().as_ref(), text_to_search.as_ref());
                    match compare.cmp(&0) {
                        Ordering::Equal => {
                            item = child;
                            not_found = false;
                            break;
                        },

                        // If it's less, we still can find the item.
                        Ordering::Less => {}

                        // If it's greater, we passed all the possible items and we can no longer find the folder.
                        Ordering::Greater => {
                            break;
                        },
                    }
                }

                // If the child was not found, stop and return the parent.
                if not_found { break; }
            }
        }

        item
    }

    unsafe fn get_root_source_type_from_selection(&self, has_filter: bool) -> Option<DataSource> {

        let filter: Option<QPtr<QSortFilterProxyModel>> = if has_filter { Some(self.model().static_downcast()) } else { None };

        let indexes_visual = self.selection_model().selection().indexes();
        let mut indexes_visual = (0..indexes_visual.count_0a()).rev().map(|x| indexes_visual.take_at(x)).collect::<Vec<CppBox<QModelIndex>>>();
        indexes_visual.reverse();
        let mut indexes_real = if let Some(filter) = filter {
            indexes_visual.iter().map(|x| filter.map_to_source(x.as_ref())).collect::<Vec<CppBox<QModelIndex>>>()
        } else {
            indexes_visual
        };

        if !indexes_real.is_empty() {
            let index = indexes_real.remove(0);
            Some(self.get_root_source_type_from_index(index))
        } else {
            None
        }
    }

    unsafe fn get_root_source_type_from_index(&self, mut index: CppBox<QModelIndex>) -> DataSource {
        let mut parent;
        let data_source;

        // Loop until we reach the root index.
        loop {

            // Get this first because, for whatever reason, once we call parent and it's invalid, the index is also invalid.
            let root_type = index.data_1a(ROOT_NODE_TYPE).to_int_0a();
            parent = index.parent();

            // If the parent is valid, it's the new item. Otherwise, we stop without adding it (we don't want the PackFile's name in).
            if parent.is_valid() {
                index = parent;
            } else {
                match root_type {
                    ROOT_NODE_TYPE_EDITABLE_PACKFILE |
                    ROOT_NODE_TYPE_NON_EDITABLE_PACKFILE => data_source = DataSource::PackFile,
                    ROOT_NODE_TYPE_PARENT_DATA => data_source = DataSource::ParentFiles,
                    ROOT_NODE_TYPE_GAME_DATA => data_source = DataSource::GameFiles,
                    ROOT_NODE_TYPE_ASSKIT => data_source = DataSource::AssKitFiles,
                    _ => unimplemented!(),
                }
                break;
            }
        }

        data_source
    }

    unsafe fn get_combination_from_main_treeview_selection(pack_file_contents_ui: &Rc<PackFileContentsUI>) -> (u8, u32, u32, u32) {

        // Get the currently selected paths, and get how many we have of each type.
        let selected_items = Self::get_item_types_from_main_treeview_selection(pack_file_contents_ui);
        let (mut file, mut folder, mut pack) = (0, 0, 0);
        let mut item_types = vec![];
        for item_type in &selected_items {
            match item_type {
                ContainerPath::File(path) => if path.is_empty() {
                    pack += 1;
                } else {
                    file += 1;
                }
                ContainerPath::Folder(path) => if path.is_empty() {
                    pack += 1;
                } else {
                    folder += 1;
                }
            }
            item_types.push(item_type);
        }

        // Now we do some bitwise magic to get what type of selection combination we have.
        let mut contents: u8 = 0;
        if file != 0 { contents |= 1; }
        if folder != 0 { contents |= 2; }
        if pack != 0 { contents |= 4; }

        (contents, file, folder, pack)
    }

    unsafe fn get_type_from_item(item: Ptr<QStandardItem>, model: &QPtr<QStandardItemModel>) -> ContainerPath {
        match item.data_1a(ITEM_TYPE).to_int_0a() {
            ITEM_TYPE_FILE => ContainerPath::File(Self::get_path_from_item(item, model)),
            ITEM_TYPE_FOLDER => ContainerPath::Folder(Self::get_path_from_item(item, model)),
            ITEM_TYPE_PACKFILE => ContainerPath::Folder(String::new()),
            _ => unreachable!()
        }
    }

    unsafe fn get_path_from_item(item: Ptr<QStandardItem>, model: &QPtr<QStandardItemModel>) -> String {
        let index = item.index();
        Self::get_path_from_index(index.as_ref(), model)
    }

    unsafe fn get_path_from_index(index: Ref<QModelIndex>, model: &QPtr<QStandardItemModel>) -> String {

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
        path.join("/")
    }

    unsafe fn get_path_from_selection(&self) -> Vec<String> {

        // Create the vector to hold the Paths and get the selected indexes of the TreeView.
        let filter: QPtr<QSortFilterProxyModel> = self.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
        let selection_model = self.selection_model();

        let mut paths: Vec<_> = vec![];
        let indexes = filter.map_selection_to_source(&selection_model.selection()).indexes();
        for index_num in 0..indexes.count_0a() {
            paths.push(Self::get_path_from_index(indexes.at(index_num), &model));
        }
        paths
    }

    unsafe fn get_path_from_pathbuf(pack_file_contents_ui: &Rc<PackFileContentsUI>, file_path: &Path, is_file: bool) -> Vec<String> {
        let mut paths = vec![];

        // If it's a single file, we get his name and push it to the paths vector.
        if is_file {
            paths.push(file_path.file_name().unwrap().to_string_lossy().as_ref().to_owned());
        }

        // Otherwise, it's a folder, so we have to filter it first.
        else {

            // Get the "Prefix" of the folder (path without the folder's name).
            let mut useless_prefix = file_path.to_path_buf();
            useless_prefix.pop();

            // Get the paths of all the files inside that folder, recursively.
            let file_list = files_from_subdir(file_path, true).unwrap();

            // Then, for each file, remove his prefix, leaving only the path from the folder onwards.
            for file_path in &file_list {
                let filtered_path = file_path.strip_prefix(&useless_prefix).unwrap();
                let filtered_path = filtered_path.to_string_lossy().replace('\\', "/");
                paths.push(filtered_path);
            }
        }

        // Then build the container paths for each file.
        for path in &mut paths {
            let selected_paths = pack_file_contents_ui.packfile_contents_tree_view().get_path_from_selection();
            let mut base_path = selected_paths[0].to_owned();

            if !base_path.ends_with('/') {
                base_path.push('/');
            }

            base_path.push_str(path);
            *path = base_path;
        }

        // Return the paths (sorted from parent to children)
        paths
    }

    unsafe fn take_row_from_path(path: &ContainerPath, model: &QPtr<QStandardItemModel>) -> Ptr<QListOfQStandardItem> {
        let is_file = matches!(path, ContainerPath::File(_));
        let path = path.path_raw().split('/').collect::<Vec<_>>();

        // First, we're effectively removing an item from the model, so mark the Pack as forever modified.
        let mut item = model.item_1a(0);
        item.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS);
        item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);

        // Looping downwards to find the item to take.
        for index in 0..path.len() {
            let name_q_string = QString::from_std_str(path[index]);

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
                if child.text().compare_q_string(&name_q_string) == 0 {
                    child.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS);
                    child.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                    item = child;
                    break;
                }
            }
        }

        // Take the child.
        item.parent().take_row(item.row()).into_ptr()
    }

    unsafe fn update_treeview(&self, has_filter: bool, operation: TreeViewOperation, source: DataSource) {
        let filter: Option<QPtr<QSortFilterProxyModel>> = if has_filter { Some(self.model().static_downcast()) } else { None };
        let model: QPtr<QStandardItemModel> = if let Some(ref filter) = filter { filter.source_model().static_downcast() } else { self.model().static_downcast() };

        // Make sure we don't try to update the view until the model is done.
        self.set_updates_enabled(false);

        // We act depending on the operation requested.
        match operation {

            // If we want to build a new TreeView...
            TreeViewOperation::Build(build_data) => {

                // Get the root node and the data to fill the rest.
                let (big_parent, mut packed_files_data) = match source {

                    // If it's a PackFile, two possibilities: editable (normal packfile) or non-editable (Add From Packfile).
                    DataSource::PackFile => {

                        // If we got data for it, use it. If not, ask the backend for it.
                        let (pack_file_data, packed_files_data) = if let Some(data) = build_data.data {
                            data
                        }
                        else if let Some(ref path) = build_data.path {
                            let receiver = CENTRAL_COMMAND.send_background(Command::GetPackFileExtraDataForTreeView(path.to_path_buf()));
                            let response = CentralCommand::recv(&receiver);
                            if let Response::ContainerInfoVecRFileInfo(data) = response { data }
                            else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); }
                        }
                        else {
                            let receiver = CENTRAL_COMMAND.send_background(Command::GetPackFileDataForTreeView);
                            let response = CentralCommand::recv(&receiver);
                            if let Response::ContainerInfoVecRFileInfo(data) = response { data }
                            else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}") }
                        };

                        // Second, we set as the big_parent, the base for the folders of the TreeView, a fake folder
                        // with the name of the PackFile. All big things start with a lie.
                        let big_parent = QStandardItem::from_q_string(&QString::from_std_str(pack_file_data.file_name()));
                        let tooltip = new_pack_file_tooltip(&pack_file_data);
                        big_parent.set_tool_tip(&QString::from_std_str(tooltip));
                        big_parent.set_editable(false);
                        big_parent.set_data_2a(&QVariant::from_int(ITEM_TYPE_PACKFILE), ITEM_TYPE);
                        big_parent.set_data_2a(&QVariant::from_int(ITEM_STATUS_PRISTINE), ITEM_STATUS);

                        if build_data.editable {
                            big_parent.set_data_2a(&QVariant::from_int(ROOT_NODE_TYPE_EDITABLE_PACKFILE), ROOT_NODE_TYPE);
                        } else {
                            big_parent.set_data_2a(&QVariant::from_int(ROOT_NODE_TYPE_NON_EDITABLE_PACKFILE), ROOT_NODE_TYPE);
                        }

                        TREEVIEW_ICONS.set_standard_item_icon(&big_parent, Some(&FileType::Pack));

                        // For PackFiles, we only allow one per view.
                        model.clear();

                        (big_parent.into_ptr(), packed_files_data)
                    },

                    DataSource::ParentFiles => {

                        // First, get the data.
                        let (_, packed_files_data) = if let Some(data) = build_data.data { data } else { unimplemented!() };

                        // Then, check if the root item we want already exits.
                        for row in 0..model.row_count_0a() {
                            let item = model.item_1a(row);
                            if item.data_1a(ROOT_NODE_TYPE).to_int_0a() == ROOT_NODE_TYPE_PARENT_DATA {
                                model.remove_rows_2a(row, 1);
                                break;
                            }
                        }

                        // Then, get the big parent item.
                        if !packed_files_data.is_empty() {

                            // Second, we set as the big_parent, the base for the folders of the TreeView, a fake folder
                            // with the name of the PackFile. All big things start with a lie.
                            let big_parent = QStandardItem::from_q_string(&qtr("dependencies_parent_files"));
                            big_parent.set_editable(false);
                            big_parent.set_data_2a(&QVariant::from_int(ITEM_TYPE_PACKFILE), ITEM_TYPE);
                            big_parent.set_data_2a(&QVariant::from_int(ITEM_STATUS_PRISTINE), ITEM_STATUS);
                            big_parent.set_data_2a(&QVariant::from_int(ROOT_NODE_TYPE_PARENT_DATA), ROOT_NODE_TYPE);

                            TREEVIEW_ICONS.set_standard_item_icon(&big_parent, Some(&FileType::Pack));

                            (big_parent.into_ptr(), packed_files_data)
                        } else {
                            self.set_updates_enabled(true);
                            return
                        }
                    },

                    DataSource::GameFiles => {

                        // First, get the data.
                        let (_, packed_files_data) = if let Some(data) = build_data.data { data } else { unimplemented!() };

                        // Then, check if the root item we want already exits.
                        for row in 0..model.row_count_0a() {
                            let item = model.item_1a(row);
                            if item.data_1a(ROOT_NODE_TYPE).to_int_0a() == ROOT_NODE_TYPE_GAME_DATA {
                                model.remove_rows_2a(row, 1);
                                break;
                            }
                        }

                        // Then, get the big parent item.
                        if !packed_files_data.is_empty() {

                            // Second, we set as the big_parent, the base for the folders of the TreeView, a fake folder
                            // with the name of the PackFile. All big things start with a lie.
                            let big_parent = QStandardItem::from_q_string(&qtr("dependencies_game_files"));
                            big_parent.set_editable(false);
                            big_parent.set_data_2a(&QVariant::from_int(ITEM_TYPE_PACKFILE), ITEM_TYPE);
                            big_parent.set_data_2a(&QVariant::from_int(ITEM_STATUS_PRISTINE), ITEM_STATUS);
                            big_parent.set_data_2a(&QVariant::from_int(ROOT_NODE_TYPE_GAME_DATA), ROOT_NODE_TYPE);

                            TREEVIEW_ICONS.set_standard_item_icon(&big_parent, Some(&FileType::Pack));

                            (big_parent.into_ptr(), packed_files_data)
                        } else {
                            self.set_updates_enabled(true);
                            return
                        }
                    },

                    DataSource::AssKitFiles => {

                        // First, get the data.
                        let (_, packed_files_data) = if let Some(data) = build_data.data { data } else { unimplemented!() };

                        // Then, check if the root item we want already exits.
                        for row in 0..model.row_count_0a() {
                            let item = model.item_1a(row);
                            if item.data_1a(ROOT_NODE_TYPE).to_int_0a() == ROOT_NODE_TYPE_ASSKIT {
                                model.remove_rows_2a(row, 1);
                                break;
                            }
                        }

                        // Then, get the big parent item.
                        if !packed_files_data.is_empty() {

                            // Second, we set as the big_parent, the base for the folders of the TreeView, a fake folder
                            // with the name of the PackFile. All big things start with a lie.
                            let big_parent = QStandardItem::from_q_string(&qtr("dependencies_asskit_files"));
                            big_parent.set_editable(false);
                            big_parent.set_data_2a(&QVariant::from_int(ITEM_TYPE_PACKFILE), ITEM_TYPE);
                            big_parent.set_data_2a(&QVariant::from_int(ITEM_STATUS_PRISTINE), ITEM_STATUS);
                            big_parent.set_data_2a(&QVariant::from_int(ROOT_NODE_TYPE_ASSKIT), ROOT_NODE_TYPE);

                            TREEVIEW_ICONS.set_standard_item_icon(&big_parent, Some(&FileType::Pack));

                            (big_parent.into_ptr(), packed_files_data)
                        } else {
                            self.set_updates_enabled(true);
                            return
                        }
                    },

                    DataSource::ExternalFile => unimplemented!()
                };

                // We sort the paths with this horrific monster I don't want to touch ever again, using the following format:
                // - FolderA
                // - FolderB
                // - FileA
                // - FileB
                sort_folders_before_files_alphabetically_file_infos(&mut packed_files_data);

                let variant_type_file = QVariant::from_int(ITEM_TYPE_FILE);
                let variant_type_folder = QVariant::from_int(ITEM_TYPE_FOLDER);
                let variant_status_pristine = QVariant::from_int(ITEM_STATUS_PRISTINE);

                let base_file_item = QStandardItem::from_q_string(&QString::new());
                base_file_item.set_editable(false);
                base_file_item.set_data_2a(&variant_type_file, ITEM_TYPE);
                base_file_item.set_data_2a(&variant_status_pristine, ITEM_STATUS);

                let base_folder_item = QStandardItem::from_q_string(&QString::new());
                base_folder_item.set_editable(false);
                base_folder_item.set_data_2a(&variant_type_folder, ITEM_TYPE);
                base_folder_item.set_data_2a(&variant_status_pristine, ITEM_STATUS);
                TREEVIEW_ICONS.set_standard_item_icon(&base_folder_item, None);

                // Once we get the entire path list sorted, we add the paths to the model one by one,
                // skipping duplicate entries.
                for packed_file in &packed_files_data {
                    let count = packed_file.path().split('/').count() - 1;

                    // First, we reset the parent to the big_parent (the PackFile).
                    // Then, we form the path ("parent -> child" style path) to add to the model.
                    let mut parent = big_parent;
                    for (index_in_path, name) in packed_file.path().split('/').enumerate() {
                        let name = QString::from_std_str(name);

                        // If it's the last string in the file path, it's a file, so we add it to the model.
                        if index_in_path == count {
                            let tooltip = new_packed_file_tooltip(packed_file);
                            let file = base_file_item.clone();
                            file.set_text(&name);

                            if !tooltip.is_empty() {
                                file.set_tool_tip(&QString::from_std_str(tooltip));
                            }

                            TREEVIEW_ICONS.set_standard_item_icon(&file, Some(packed_file.file_type()));

                            parent.append_row_q_standard_item(file);
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
                            let children_len = parent.row_count();

                            if parent.has_children() {

                                // It's a folder, so we check his children. We are only interested in
                                // folders, so ignore the files. Reverse because due to the sorting it's almost
                                // sure the last folder is the one we want.
                                for index in (0..children_len).rev() {
                                    let child = parent.child_2a(index, 0);
                                    if child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { continue }

                                    // Get his text. If it's the same folder we are trying to add, this is our parent now.
                                    let compare = child.text().compare_q_string(&name);
                                    match compare.cmp(&0) {
                                        Ordering::Equal => {
                                            parent = parent.child_1a(index);
                                            duplicate_found = true;
                                            break;
                                        },

                                        // Optimization: We get the paths pre-sorted. If the last folder cannot be under our folder, stop iterating.
                                        Ordering::Less => {
                                            break;
                                        },
                                        Ordering::Greater => {},
                                    }
                                }
                            }

                            // If our current parent doesn't have anything, just add it as a new folder.
                            if !duplicate_found {
                                let folder = base_folder_item.clone();
                                folder.set_text(&name);

                                parent.append_row_q_standard_item(folder);

                                // This is our parent now.
                                parent = parent.child_1a(children_len);
                            }
                        }
                    }
                }

                // Delay adding the big parent as much as we can, as otherwise the signals triggered when adding a file can slow this down to a crawl.
                model.append_row_q_standard_item(big_parent);
            },

            // If we want to add a file/folder to the `TreeView`...
            //
            // BIG NOTE: This only works for files OR EMPTY FOLDERS. If you want to add a folder with files,
            // add his files individually, not the folder!!!
            TreeViewOperation::Add(mut item_types) => {

                // Make sure all items are pre-sorted. This can speed up adding large amounts of items.
                sort_folders_before_files_alphabetically_container_paths(&mut item_types);

                // Get the `RFileInfo` of each of the new paths, so we can later build their tooltip.
                let item_paths = item_types.par_iter().map(|item| item.path_raw().to_owned()).collect::<Vec<_>>();
                let receiver = CENTRAL_COMMAND.send_background(Command::GetPackedFilesInfo(item_paths));
                let response = CentralCommand::recv(&receiver);
                let files_info = if let Response::VecRFileInfo(data) = response { data } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); };

                // Mark the base Pack as modified and having received additions.
                if !item_types.is_empty() {
                    let parent = model.item_1a(0);
                    match parent.data_1a(ITEM_STATUS).to_int_0a() {
                         ITEM_STATUS_PRISTINE => parent.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED), ITEM_STATUS),
                         ITEM_STATUS_MODIFIED => parent.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED | ITEM_STATUS_MODIFIED), ITEM_STATUS),
                         ITEM_STATUS_ADDED | 3 => {},
                         _ => unimplemented!(),
                    }

                    // We cannot revert file additions.
                    if !parent.data_1a(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                        parent.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                    }
                }

                // Add each item type, together with its own info.
                for item_type in &item_types {
                    let is_file = matches!(item_type, ContainerPath::File(_));
                    let path = item_type.path_raw();
                    let count = path.split('/').count() - 1;
                    let mut parent = model.item_1a(0);

                    for (index, name) in path.split('/').enumerate() {
                        let name_q_string = QString::from_std_str(name);

                        // If it's the last element of the path, it's a file or an empty folder. First, we check if it
                        // already exists. If it does, then we update it and set it as added. If it doesn't, we create it.
                        let mut duplicate_found = false;
                        if index >= count {

                            // If the current parent has at least one child, check if it already contains what we're trying to add.
                            if parent.has_children() {

                                // Optimization: We do it in reverse because, due to already having the paths to add pre-sorted,
                                // it's way faster to start searching for them from the end.
                                for index in (0..parent.row_count()).rev() {
                                    let child = parent.child_2a(index, 0);

                                    // We ignore files or folders, depending on what we want to create.
                                    if is_file && child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FOLDER { continue }
                                    if !is_file && child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { continue }

                                    // Get his text. If it's the same file/folder we are trying to add, this is the one.
                                    let compare = child.text().compare_q_string(name_q_string.as_ref());
                                    match compare.cmp(&0) {
                                        Ordering::Equal => {
                                            parent = parent.child_1a(index);
                                            duplicate_found = true;
                                            break;
                                        },

                                        // If our file/folder should be after this one in sorting, take it as that the file/folder doesn't exists.
                                        Ordering::Less => {
                                            break;
                                        },
                                        Ordering::Greater => {},
                                    }
                                }
                            }

                            // If the item already exist, re-use it.
                            if duplicate_found {
                                parent.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED), ITEM_STATUS);
                                parent.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                            }

                            // Otherwise, it's a new item, so we create it.
                            else {
                                let item = QStandardItem::from_q_string(&name_q_string).into_ptr();
                                item.set_editable(false);

                                if item_type.is_file() {
                                    item.set_data_2a(&QVariant::from_int(ITEM_TYPE_FILE), ITEM_TYPE);

                                    if let Some(file_info) = files_info.par_iter().find_first(|x| x.path() == item_type.path_raw()) {
                                        TREEVIEW_ICONS.set_standard_item_icon(&item, Some(file_info.file_type()));
                                        let tooltip = new_packed_file_tooltip(file_info);
                                        if !tooltip.is_empty() {
                                            item.set_tool_tip(&QString::from_std_str(tooltip));
                                        }
                                    }
                                }

                                else {
                                    item.set_data_2a(&QVariant::from_int(ITEM_TYPE_FOLDER), ITEM_TYPE);
                                    TREEVIEW_ICONS.set_standard_item_icon(&item, None);
                                }

                                item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                                item.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED), ITEM_STATUS);

                                parent.append_row_q_standard_item(item);

                                sort_item_in_tree_view(&model, item, item_type);
                            }
                        }

                        // Otherwise, it's a folder.
                        else {

                            // If the current parent has at least one child, check if the folder already exists.
                            if parent.has_children() {

                                // It's a folder, so we check his children starting by the beginning.
                                for index in 0..parent.row_count() {
                                    let child = parent.child_2a(index, 0);
                                    if child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE {
                                        break;
                                    }

                                    let compare = child.text().compare_q_string(&name_q_string);
                                    match compare.cmp(&0) {
                                        Ordering::Equal => {
                                            parent = parent.child_1a(index);
                                            match parent.data_1a(ITEM_STATUS).to_int_0a() {
                                                 ITEM_STATUS_PRISTINE => parent.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED), ITEM_STATUS),
                                                 ITEM_STATUS_MODIFIED => parent.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED | ITEM_STATUS_MODIFIED), ITEM_STATUS),
                                                 ITEM_STATUS_ADDED | 3 => {},
                                                 _ => unimplemented!(),
                                            }

                                            if !parent.data_1a(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                                                parent.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                                            }

                                            duplicate_found = true;
                                            break;
                                        },

                                        // If our file/folder should be after this one in sorting, take it as that the file/folder doesn't exists.
                                        Ordering::Greater => {
                                            break;
                                        },
                                        Ordering::Less => {},
                                    }
                                }
                            }

                            // If the folder doesn't already exists, just add it.
                            if !duplicate_found {
                                let folder = QStandardItem::from_q_string(&name_q_string).into_ptr();
                                folder.set_editable(false);
                                folder.set_data_2a(&QVariant::from_int(ITEM_TYPE_FOLDER), ITEM_TYPE);
                                folder.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED), ITEM_STATUS);
                                folder.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                                TREEVIEW_ICONS.set_standard_item_icon(&folder, None);

                                parent.append_row_q_standard_item(folder);

                                let index = parent.row_count() - 1;
                                parent = parent.child_1a(index);

                                sort_item_in_tree_view(&model, parent, &ContainerPath::Folder(String::new()));
                            }
                        }
                    }

                    if setting_bool("expand_treeview_when_adding_items") {
                        self.expand_treeview_to_item(path, source);
                    }
                }
            },

            // If we want to delete something from the TreeView.
            TreeViewOperation::Delete(paths) => {
                let paths = ContainerPath::dedup(&paths);
                let pack = model.item_1a(0);

                for path_type in paths {
                    let mut item = pack;
                    let path = path_type.path_raw();
                    let count = path.split('/').count();
                    let is_file = matches!(path_type, ContainerPath::File(_));

                    // If path is empty, it's the Pack. In this case we just rebuild the TreeView.
                    if path.is_empty() {
                        let mut build_data = BuildData::new();
                        build_data.editable = true;
                        self.update_treeview(true, TreeViewOperation::Build(build_data), source);
                    }

                    // Otherwise, it's either a file or a folder.
                    else {
                        for (index, name) in path.split('/').enumerate() {
                            let name_q_string = QString::from_std_str(name);

                            // If we reached the final element of the path, try to find it on the children of the current parent.
                            if index == count - 1 {
                                for row in 0..item.row_count() {
                                    let child = item.child_1a(row);

                                    if is_file && child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FOLDER { continue }
                                    if !is_file && child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { continue }

                                    // If we found it, we're done.
                                    if child.text().compare_q_string(&name_q_string) == 0 {
                                        item = child;
                                        break;
                                    }
                                }
                            }

                            // If we are not still in the final folder, we only look for a folder.
                            else {
                                for row in 0..item.row_count() {
                                    let child = item.child_1a(row);
                                    if child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { continue }

                                    // If we found one with children, check if it's the one we want. If it is, that's out new good boy.
                                    if child.text().compare_q_string(&name_q_string) == 0 {
                                        item = child;
                                        break;
                                    }
                                }
                            }
                        }

                        // Begin the endless cycle of war and dead.
                        let mut index = 0;
                        for i in 0..count {

                            // Get the parent of the item, and kill the item in a cruel way.
                            index = i;
                            let parent = item.parent();

                            // Not sure what the fuck causes this, but sometimes parent is null.
                            if parent.is_null() {
                                error!("Parent null passed for path {:?}. Breaking loop to avoid crash (god knows what will happen next).", path);
                                break;
                            }
                            parent.remove_row(item.row());

                            parent.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS);
                            if !parent.data_1a(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                                parent.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                            }

                            // If the parent has more children, or we reached the PackFile, we're done. Otherwise, we update our item.
                            item = parent;
                            if parent.has_children() || !pack.has_children() {
                                break;
                            }
                        }

                        // Mark all the parents left, up to the Pack.
                        for _ in 0..index {
                            if !item.is_null() {
                                item.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS);
                                item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                                item = item.parent();
                            }
                        }
                    }
                }
            },

            // If you want to modify the contents of something...
            TreeViewOperation::Modify(path_types) => {
                for path_type in path_types {
                    let path = path_type.path_raw();
                    let path_split = path.split('/').collect::<Vec<_>>();
                    let item = Self::item_from_path(&path_type, &model);
                    match item.data_1a(ITEM_STATUS).to_int_0a() {
                        ITEM_STATUS_PRISTINE => item.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS),
                        ITEM_STATUS_ADDED => item.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED | ITEM_STATUS_MODIFIED), ITEM_STATUS),
                        ITEM_STATUS_MODIFIED | 3 => {},
                        _ => unimplemented!(),
                    };

                    // If its a file, we get his new info and put it in a tooltip.
                    if path_type.is_file() {
                        let receiver = CENTRAL_COMMAND.send_background(Command::GetRFileInfo(path.to_owned()));
                        let response = CentralCommand::recv(&receiver);
                        let packed_file_info = if let Response::OptionRFileInfo(data) = response { data } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); };
                        if let Some(info) = packed_file_info {
                            let tooltip = new_packed_file_tooltip(&info);
                            if !tooltip.is_empty() {
                                item.set_tool_tip(&QString::from_std_str(tooltip));
                            }
                        }
                    }

                    let cycles = if !path_split.is_empty() { path_split.len() - 1 } else { 0 };
                    let mut parent = item.parent();
                    for _ in 0..cycles {

                        // Get the status and mark them as needed.
                        match parent.data_1a(ITEM_STATUS).to_int_0a() {
                            ITEM_STATUS_PRISTINE => parent.set_data_2a(&QVariant::from_int(ITEM_STATUS_MODIFIED), ITEM_STATUS),
                            ITEM_STATUS_ADDED => parent.set_data_2a(&QVariant::from_int(ITEM_STATUS_ADDED | ITEM_STATUS_MODIFIED), ITEM_STATUS),
                            ITEM_STATUS_MODIFIED | 3 => {},
                            _ => unimplemented!(),
                        };

                        // Set the new parent.
                        parent = parent.parent();
                    }
                }
            }

            // If we want to move something from one point of the view to another.
            // In case the new path doesn't exists, it's created.
            // Unlike delete, this doesn't remove empty folders unless its the folder we moved..
            TreeViewOperation::Move(moved_paths, base_folders) => {

                // First, get the `RFileInfo` of each of the new paths (so we can later build their tooltip, if neccesary).
                // Only needed for files, ignore folders on this one.
                let new_paths = moved_paths.iter()
                    .filter_map(|(_, y)| if let ContainerPath::File(path) = y { Some(path.to_owned()) } else { None })
                    .collect::<Vec<String>>();

                let receiver = CENTRAL_COMMAND.send_background(Command::GetPackedFilesInfo(new_paths));
                let response = CentralCommand::recv(&receiver);
                let files_info = if let Response::VecRFileInfo(data) = response { data } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); };

                for (source_path, new_path) in &moved_paths  {
                    let taken_row = Self::take_row_from_path(source_path, &model);

                    let new_path = match new_path {
                        ContainerPath::File(path) => path,
                        ContainerPath::Folder(path) => path,
                    };

                    // TODO: This may be slow on big moves. Fix it with a hashmap.
                    let file_info = files_info.iter().find(|file_info| file_info.path() == new_path);
                    Self::add_row_to_path(taken_row, &model, new_path, file_info);
                    self.expand_treeview_to_item(new_path, source);
                }

                // Remove the now empty folders.
                self.update_treeview(has_filter, TreeViewOperation::Delete(base_folders), source);
            },

            // If you want to mark an item so it can't lose his modified state...
            TreeViewOperation::MarkAlwaysModified(item_types) => {
                for item_type in &item_types {
                    let item = Self::item_from_path(item_type, &model);
                    if !item.is_null() && !item.data_1a(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                        item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_FOREVER_MODIFIED);
                    }
                }
            }

            // If we want to undo the doings of any PackFile.
            TreeViewOperation::Undo(item_types) => {
                for item_type in item_types {
                    match item_type {
                        ContainerPath::File(ref path) | ContainerPath::Folder(ref path) => {

                            // Get the item and only try to restore it if we didn't set it as "not to restore".
                            let item = Self::item_from_path(&item_type, &model);
                            if !item.data_1a(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                                if item.data_1a(ITEM_STATUS).to_int_0a() != ITEM_STATUS_PRISTINE {
                                    item.set_data_2a(&QVariant::from_int(ITEM_STATUS_PRISTINE), ITEM_STATUS);
                                }

                                // If its a file, we get his new info and put it in a tooltip.
                                if let ContainerPath::File(_) = item_type {
                                    let receiver = CENTRAL_COMMAND.send_background(Command::GetRFileInfo(path.to_owned()));
                                    let response = CentralCommand::recv(&receiver);
                                    let packed_file_info = if let Response::OptionRFileInfo(data) = response { data } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); };
                                    if let Some(info) = packed_file_info {
                                        let tooltip = new_packed_file_tooltip(&info);
                                        if !tooltip.is_empty() {
                                            item.set_tool_tip(&QString::from_std_str(tooltip));
                                        }
                                    }
                                }

                                // Get the times we must to go up until we reach the parent.
                                let cycles = if !path.is_empty() { path.len() } else { 0 };
                                let mut parent = item.parent();

                                // Unleash hell upon the land.
                                for _ in 0..cycles {

                                    if !parent.data_1a(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                                        if parent.data_1a(ITEM_STATUS).to_int_0a() != ITEM_STATUS_PRISTINE {
                                            parent.set_data_2a(&QVariant::from_int(ITEM_STATUS_PRISTINE), ITEM_STATUS);
                                        }
                                        else { break; }
                                    }
                                    else { break; }
                                    parent = parent.parent();
                                }
                            }
                        }

                        // This one is a bit special. We need to check, not only him, but all his children too.
                        /*ContainerPath::PackFile => {
                            let item = model.item_2a(0, 0);
                            let mut packfile_is_modified = false;
                            for row in 0..item.row_count() {
                                let child = item.child_2a(row, 0);
                                if child.data_1a(ITEM_STATUS).to_int_0a() != ITEM_STATUS_PRISTINE {
                                    packfile_is_modified = true;
                                    break;
                                }
                            }

                            if !packfile_is_modified {
                                item.set_data_2a(&QVariant::from_int(ITEM_STATUS_PRISTINE), ITEM_STATUS);
                            }
                        }*/
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
                    let tooltip = new_packed_file_tooltip(&packed_file_info);
                    if !tooltip.is_empty() {
                        let tooltip = QString::from_std_str(tooltip);
                        let tree_path_type = ContainerPath::File(packed_file_info.path().to_owned());
                        let item = Self::item_from_path(&tree_path_type, &model);
                        item.set_tool_tip(&tooltip);
                    }
                }
            },
        }

        // Re-enable the view.
        self.set_updates_enabled(true);
    }
}

//----------------------------------------------------------------//
// Helpers to control the main TreeView.
//----------------------------------------------------------------//

/// This function is used to create the tooltip for the `PackFile` item in the PackFile Content's TreeView.
pub fn new_pack_file_tooltip(info: &ContainerInfo) -> String {
    format!("Pack Info: \
        <ul> \
            <li><b>PFH Version:</b> <i>{}</i></li> \
            <li><b>Is Encrypted:</b> <i>{}</i></li> \
            <li><b>Last Modified:</b> <i>{}</i></li> \
        </ul>",
        info.pfh_version(),
        info.bitmask().contains(PFHFlags::HAS_ENCRYPTED_INDEX) || info.bitmask().contains(PFHFlags::HAS_ENCRYPTED_DATA),
        OffsetDateTime::from_unix_timestamp(*info.timestamp() as i64)
            .unwrap()
            .format(&FULL_DATE_FORMAT)
            .unwrap()
    )
}

/// This function is used to create the tooltip for each `PackedFile` item in the PackFile Content's TreeView.
fn new_packed_file_tooltip(info: &RFileInfo) -> String {
    let mut string = String::from("File Info: <ul>");
    let mut has_info = false;

    if let Some(container_name) = info.container_name() {
        string.push_str(&format!("<li><b>Original Pack:</b> <i>{container_name}</i></li>"));
        has_info = true;
    }

    if let Some(timestamp) = info.timestamp() {
        string.push_str(&format!("<li><b>Last Modified:</b> <i>{}</i></li>", OffsetDateTime::from_unix_timestamp(*timestamp as i64)
            .unwrap()
            .format(&FULL_DATE_FORMAT)
            .unwrap()));
        has_info = true;
    }

    if has_info {
        string.push_str("</ul>");
        string
    } else {
        String::new()
    }
}

/// This function cleans the entire TreeView from colors. To be used when saving.
unsafe fn clean_treeview(item: Option<Ptr<QStandardItem>>, model: &QStandardItemModel) {

    // Only do it if the model actually have something.
    if model.row_count_0a() > 0 {

        // If we receive None, use the PackFile.
        let item = if let Some(item) = item { item } else { model.item_2a(0, 0) };

        // Clean the current item, and repeat for each children.
        item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_FOREVER_MODIFIED);
        item.set_data_2a(&QVariant::from_int(ITEM_STATUS_PRISTINE), ITEM_STATUS);

        let children_count = item.row_count();
        for row in 0..children_count {
            let child = item.child_2a(row, 0);
            clean_treeview(Some(child), model);
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
    item_type: &ContainerPath,
) {

    // Get the index of our item, and our item's parent index.
    let mut item_index = item.index();
    let parent = item.parent();
    let parent_index = parent.index();

    // Get the previous and next indexes on the list.
    let item_index_prev = model.index_3a(item_index.row() - 1, item_index.column(), &parent_index);
    let item_index_next = model.index_3a(item_index.row() + 1, item_index.column(), &parent_index);

    // Get the type of the previous item on the list.
    let item_type_prev: Option<ContainerPath> = if item_index_prev.is_valid() {
        let item_sibling = model.item_from_index(&item_index_prev);
        Some(<QPtr<QTreeView>>::get_type_from_item(item_sibling, model))
    } else { None };

    // Get the type of the next item on the list.
    let item_type_next: Option<ContainerPath> = if item_index_next.is_valid() {
        let item_sibling = model.item_from_index(&item_index_next);
        Some(<QPtr<QTreeView>>::get_type_from_item(item_sibling, model))
    } else { None };

    // We get the boolean to determinate the direction to move (true -> up, false -> down).
    // If the previous and the next items are `None`, we don't need to move as there are no more items.
    let direction = if item_type_prev.is_none() && item_type_next.is_none() { return }

    // If the top one is `None`, but the bottom one isn't, we go down.
    else if item_type_prev.is_none() && item_type_next.is_some() { false }

    // If the bottom one is `None`, but the top one isn't, we go up.
    else if item_type_prev.is_some() && item_type_next.is_none() { true }

    // If the top one is a folder, and the bottom one is a file, act depending on the type of our item.
    else if item_type_prev.unwrap().is_folder() && item_type_next.unwrap().is_file() {
        *item_type == ContainerPath::Folder(String::new())
    }

    // If the two around it are the same type, compare them and decide.
    else {

        // Get the previous, current and next texts.
        let previous_name = parent.child_1a(item_index.row() - 1).text();
        let current_name = parent.child_1a(item_index.row()).text();
        let next_name = parent.child_1a(item_index.row() + 1).text();

        let compare_prev = previous_name.compare_q_string(&current_name);
        let compare_next = next_name.compare_q_string(&current_name);

        // If we don't need to move, just return.
        if compare_prev < 0 && compare_next > 0 {
            return;
        } else {
            compare_prev > 0
        }
    };

    // We "sort" it among his peers.
    loop {

        // Get the previous and next item ModelIndex on the list.
        let item_index_prev = item_index.sibling(item_index.row() - 1, 0);
        let item_index_next = item_index.sibling(item_index.row() + 1, 0);

        // Depending on the direction we have to move, get the second item's index.
        let item_sibling_index = if direction { item_index_prev } else { item_index_next };
        if item_sibling_index.is_valid() {

            // Get the Item sibling to our current Item.
            let item_sibling = parent.child_1a(item_sibling_index.row());
            let item_sibling_type = <QPtr<QTreeView>>::get_type_from_item(item_sibling, model);

            // If both are of the same type...
            if item_type.is_file() == item_sibling_type.is_file() || item_type.is_folder() == item_sibling_type.is_folder() {

                // Get both texts.
                let item_name = item.text();
                let sibling_name = item_sibling.text();
                let compare = item_name.compare_q_string(&sibling_name);

                // Depending on our direction, we sort one way or another
                if direction {
                    match compare.cmp(&0) {

                        // This means we need to move our item up.
                        Ordering::Less => {
                            let item_x = parent.take_row(item_index.row());
                            parent.insert_row_int_q_list_of_q_standard_item(item_sibling_index.row(), &item_x);
                            item = parent.child_1a(item_sibling_index.row());
                            item_index = item.index();
                        },

                        // This cannot happen unless someone else bug out a Pack.
                        Ordering::Equal => { dbg!("bug"); break; },

                        // This means we reached our intended position.
                        Ordering::Greater => {
                            break;
                        },
                    }
                } else {
                    match compare.cmp(&0) {

                        // This means we reached our intended position.
                        Ordering::Less => {
                            break;
                        },

                        // This cannot happen unless someone else bug out a Pack.
                        Ordering::Equal => { dbg!("bug"); break; },

                        // This means we need to move our item up.
                        Ordering::Greater => {
                            let item_x = parent.take_row(item_index.row());
                            parent.insert_row_int_q_list_of_q_standard_item(item_sibling_index.row(), &item_x);
                            item = parent.child_1a(item_sibling_index.row());
                            item_index = item.index();
                        },
                    }
                }
            }

            // If the top one is a File and the bottom one a Folder, it's an special situation. Just swap them.
            else if item_type.is_folder() && item_sibling_type.is_file() {

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

/// Implementation of `BuildData`.
impl BuildData {

    /// This function creates a new build data for a non-editable PackFile.
    pub fn new() -> Self {
        Self {
            path: None,
            data: None,
            editable: false,
        }
    }
}

fn sort_folders_before_files_alphabetically_container_paths(files: &mut Vec<ContainerPath>) {
    files.par_sort_unstable_by(|a, b| {
        let a_path = a.path_raw();
        let b_path = b.path_raw();

        sort_folders_before_files_alphabetically_paths(a_path, b_path)
    });
}

// We sort the paths with this horrific monster I don't want to touch ever again, using the following format:
// - FolderA
// - FolderB
// - FileA
// - FileB
fn sort_folders_before_files_alphabetically_file_infos(files: &mut Vec<RFileInfo>) {
    files.par_sort_unstable_by(|a, b| {
        let a_path = a.path();
        let b_path = b.path();

        sort_folders_before_files_alphabetically_paths(a_path, b_path)
    });
}

fn sort_folders_before_files_alphabetically_paths(a_path: &str, b_path: &str) -> Ordering {
    let mut a_iter = a_path.rmatch_indices('/');
    let mut b_iter = b_path.rmatch_indices('/');

    let (a_last_split, a_len) = {
        match a_iter.next() {
            Some((index, _)) => (index, a_iter.count() + 1),
            None => (0, 0),
        }
    };
    let (b_last_split, b_len) = {
        match b_iter.next() {
            Some((index, _)) => (index, b_iter.count() + 1),
            None => (0, 0),
        }
    };

    // Short-circuit cases: one or both files on root.
    if a_last_split == 0 && b_last_split == 0 {
        return a_path.cmp(b_path);
    } else if a_last_split == 0 {
        return Ordering::Greater;
    } else if b_last_split == 0 {
        return Ordering::Less;
    }

    // Short-circuit: both are files under the same amount of subfolders.
    if a_len == b_len {
        a_path.cmp(b_path)
    } else if a_len > b_len {
        if a_path.starts_with(&b_path[..b_last_split]) {
            Ordering::Less
        } else {
            a_path.cmp(b_path)
        }
    } else if b_path.starts_with(&a_path[..a_last_split]) {
        Ordering::Greater
    } else {
        a_path.cmp(b_path)
    }
}

pub unsafe fn get_color_correct() -> String {
    if setting_bool("use_dark_theme") {
        GREEN_DARK.to_owned()
    } else {
        GREEN_BRIGHT.to_owned()
    }
}

pub unsafe fn get_color_wrong() -> String {
    if setting_bool("use_dark_theme") {
        RED_DARK.to_owned()
    } else {
        RED_BRIGHT.to_owned()
    }
}

pub unsafe fn get_color_clean() -> String {
    if setting_bool("use_dark_theme") {
        MEDIUM_DARKER_GREY.to_owned()
    } else {
        TRANSPARENT_BRIGHT.to_owned()
    }
}

pub unsafe fn get_color_info() -> String {
    if setting_bool("use_dark_theme") {
        INFO_UNPRESSED_DARK.to_owned()
    } else {
        INFO_UNPRESSED_LIGHT.to_owned()
    }
}

pub unsafe fn get_color_warning() -> String {
    if setting_bool("use_dark_theme") {
        WARNING_UNPRESSED_DARK.to_owned()
    } else {
        WARNING_UNPRESSED_LIGHT.to_owned()
    }
}

pub unsafe fn get_color_error() -> String {
    if setting_bool("use_dark_theme") {
        ERROR_UNPRESSED_DARK.to_owned()
    } else {
        ERROR_UNPRESSED_LIGHT.to_owned()
    }
}

pub unsafe fn get_color_info_pressed() -> String {
    if setting_bool("use_dark_theme") {
        INFO_PRESSED_DARK.to_owned()
    } else {
        INFO_PRESSED_LIGHT.to_owned()
    }
}

pub unsafe fn get_color_warning_pressed() -> String {
    if setting_bool("use_dark_theme") {
        WARNING_PRESSED_DARK.to_owned()
    } else {
        WARNING_PRESSED_LIGHT.to_owned()
    }
}

pub unsafe fn get_color_error_pressed() -> String {
    if setting_bool("use_dark_theme") {
        ERROR_PRESSED_DARK.to_owned()
    } else {
        ERROR_PRESSED_LIGHT.to_owned()
    }
}
