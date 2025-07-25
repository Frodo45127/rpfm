//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code for the ESFTree, the tree used for the ESF Views.

It's similar to the PackTree, but modified for the requirements of the ESF files.
!*/

use qt_widgets::q_header_view::ResizeMode;
use qt_widgets::QTreeView;

use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;
use qt_gui::QListOfQStandardItem;

use qt_core::ItemFlag;
use qt_core::QFlags;
use qt_core::QPtr;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QVariant;

use cpp_core::CppBox;
use cpp_core::Ptr;

use rpfm_lib::files::esf::{ESF, NodeType, RecordNodeFlags};

const ESF_DATA: i32 = 40;
const CHILDLESS_NODE: i32 = 41;
const CHILD_NODES: i32 = 42;
const RECORD_NODE_NAME: i32 = 43;

//-------------------------------------------------------------------------------//
//                          Enums & Structs (and trait)
//-------------------------------------------------------------------------------//

/// This trait adds multiple util functions to the `TreeView` you implement it for.
///
/// Keep in mind that this trait has been created with `ESF TreeView's` in mind, so his methods
/// may not be suitable for all purposes.
pub(crate) trait ESFTree {

    /// This function generates an ESF file from the contents of the `TreeView`.
    unsafe fn get_esf_from_view(&self, has_filter: bool) -> ESF;

    /// This function gives you the data contained within a CHILD_NODES variant of the provided item.
    unsafe fn get_child_nodes_from_item(item: &QStandardItem) -> String;

    /// This function takes care of EVERY operation that manipulates the provided TreeView.
    /// It does one thing or another, depending on the operation we provide it.
    unsafe fn update_treeview(&self, has_filter: bool, operation: ESFTreeViewOperation);
}

/// This enum has the different possible operations we can do in a `TreeView`.
#[derive(Clone, Debug)]
pub enum ESFTreeViewOperation<'a> {

    /// Build the entire `TreeView` from the provided ESF data.
    Build(&'a ESF),
}

//-------------------------------------------------------------------------------//
//                      Implementations of `ESFTree`
//-------------------------------------------------------------------------------//

/// Implementation of `ESFTree` for `QPtr<QTreeView>`.
impl ESFTree for QPtr<QTreeView> {

    unsafe fn get_child_nodes_from_item(item: &QStandardItem) -> String {
        item.data_1a(CHILD_NODES).to_string().to_std_string()
    }

    unsafe fn update_treeview(&self, has_filter: bool, operation: ESFTreeViewOperation) {
        let filter: Option<QPtr<QSortFilterProxyModel>> = if has_filter { Some(self.model().static_downcast()) } else { None };
        let model: QPtr<QStandardItemModel> = if let Some(ref filter) = filter { filter.source_model().static_downcast() } else { self.model().static_downcast() };

        // We act depending on the operation requested.
        match operation {

            // If we want to build a new TreeView...
            ESFTreeViewOperation::Build(esf_data) => {

                // First, we clean the TreeStore and whatever was created in the TreeView.
                model.clear();

                // Second, we set as the big_parent, the base for the folders of the TreeView, a fake folder
                // with the name of the PackFile. All big things start with a lie.
                let root_node = esf_data.root_node();
                if let NodeType::Record(node) = root_node {

                    let big_parent = QStandardItem::from_q_string(&QString::from_std_str(node.name()));
                    let state_item = QStandardItem::new();
                    big_parent.set_editable(false);
                    state_item.set_editable(false);
                    state_item.set_selectable(false);

                    let esf_data_no_node: ESF = esf_data.clone_without_root_node();
                    big_parent.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&esf_data_no_node).unwrap())), ESF_DATA);
                    big_parent.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&root_node.clone_without_children()).unwrap())), CHILDLESS_NODE);
                    big_parent.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&node.children()[0].iter().map(|x| x.clone_without_children()).collect::<Vec<NodeType>>()).unwrap())), CHILD_NODES);
                    big_parent.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(node.name())), RECORD_NODE_NAME);

                    let flags = ItemFlag::from(state_item.flags().to_int() & ItemFlag::ItemIsSelectable.to_int());
                    state_item.set_flags(QFlags::from(flags));

                    for node_group in node.children() {
                        for node in node_group {
                            load_node_to_view(&big_parent, node, None);
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
                }
            },
        }
    }

    unsafe fn get_esf_from_view(&self, has_filter: bool) -> ESF {
        let filter: Option<QPtr<QSortFilterProxyModel>> = if has_filter { Some(self.model().static_downcast()) } else { None };
        let model: QPtr<QStandardItemModel> = if let Some(ref filter) = filter { filter.source_model().static_downcast() } else { self.model().static_downcast() };

        let mut new_esf: ESF = serde_json::from_str(&model.item_1a(0).data_1a(ESF_DATA).to_string().to_std_string()).unwrap();
        new_esf.set_root_node(get_node_type_from_tree_node(None, &model));

        // Return the created ESF.
        // TODO: check this returns the exact same ESF if there are no changes.
        new_esf
    }
}

/// This function takes care of recursively loading all the nodes into the `TreeView`.
unsafe fn load_node_to_view(parent: &CppBox<QStandardItem>, child: &NodeType, block_key: Option<&str>) {
    if let NodeType::Record(node) = child {

        // Create the node for the record.
        let child_item = QStandardItem::from_q_string(&QString::from_std_str(node.name()));
        let state_item = QStandardItem::new();
        child_item.set_editable(false);
        state_item.set_editable(false);
        state_item.set_selectable(false);

        // If it has a name (it should have it), name it.
        if let Some(block_key) = block_key {
            child_item.set_text(&QString::from_std_str(block_key));
        }

        // Prepare the data in a way or another, depending if we have nested blocks or not.
        if node.record_flags().contains(RecordNodeFlags::HAS_NESTED_BLOCKS) {
            for (index, node_group) in node.children().iter().enumerate() {
                let node_group_name = if node_group.len() == 2 {
                    if let NodeType::Ascii(ref key) = node_group[0] {
                        key.to_owned()
                    } else { format!("{}_{}", node.name(), index) }
                } else { format!("{}_{}", node.name(), index) };
                let node_group_item = QStandardItem::from_q_string(&QString::from_std_str(&node_group_name));
                let node_group_state_item = QStandardItem::new();
                node_group_item.set_editable(false);
                node_group_state_item.set_editable(false);
                node_group_state_item.set_selectable(false);

                // Put all record nodes under the "Group Node".
                for grandchild_node in node_group {
                    if let NodeType::Record(_) = grandchild_node {
                        load_node_to_view(&node_group_item, grandchild_node, None);
                    }
                }

                // Store the group's data.
                node_group_item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&node_group.iter().map(|x| x.clone_without_children()).collect::<Vec<NodeType>>()).unwrap())), CHILD_NODES);
                node_group_item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(&node_group_name)), RECORD_NODE_NAME);

                let qlist = QListOfQStandardItem::new();
                qlist.append_q_standard_item(&node_group_item.into_ptr().as_mut_raw_ptr());
                qlist.append_q_standard_item(&node_group_state_item.into_ptr().as_mut_raw_ptr());

                child_item.append_row_q_list_of_q_standard_item(qlist.as_ref());
            }

            // Set the child's data, and add the child to the TreeView.
            child_item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&child.clone_without_children()).unwrap())), CHILDLESS_NODE);
            child_item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(node.name())), RECORD_NODE_NAME);
        }

        // If it doesn't have nested blocks, just grab the first block's pack.
        else {

            // First, load record nodes into the view.
            for child_node in &node.children()[0] {
                if let NodeType::Record(_) = child_node {
                    load_node_to_view(&child_item, child_node, None);
                }
            }

            // Once done, store its data and it's values.
            child_item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&child.clone_without_children()).unwrap())), CHILDLESS_NODE);
            child_item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&node.children()[0].iter().map(|x| x.clone_without_children()).collect::<Vec<NodeType>>()).unwrap())), CHILD_NODES);
            child_item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(node.name())), RECORD_NODE_NAME);
        }

        let qlist = QListOfQStandardItem::new();
        qlist.append_q_standard_item(&child_item.into_ptr().as_mut_raw_ptr());
        qlist.append_q_standard_item(&state_item.into_ptr().as_mut_raw_ptr());

        parent.append_row_q_list_of_q_standard_item(qlist.as_ref());
    }
}

/// This function reads the entire `TreeView` recursively and returns a node list.
unsafe fn get_node_type_from_tree_node(current_item: Option<Ptr<QStandardItem>>, model: &QStandardItemModel) -> NodeType {

    // Try to get the node info. If it fails, this node is not a proper node, but a child of a node.
    let item = if let Some(item) = current_item { item } else { model.item_1a(0) };
    let mut node = serde_json::from_str(&item.data_1a(CHILDLESS_NODE).to_string().to_std_string()).unwrap();

    // If it has no children, just its json.
    match node {
        NodeType::Record(ref mut node) => {

            // Depending if we should have nested blocks or not, get the children in one way or another.
            if node.record_flags().contains(RecordNodeFlags::HAS_NESTED_BLOCKS) {

                // Get the record group nodes, and process the groups one by one.
                let record_group_count = item.row_count();
                let mut record_group_nodes = Vec::with_capacity(record_group_count as usize);
                for row in 0..record_group_count {

                    let child = item.child_1a(row);
                    let child_nodes = child.data_1a(CHILD_NODES).to_string().to_std_string();
                    let mut child_nodes: Vec<NodeType> = if !child_nodes.is_empty() {
                        match serde_json::from_str(&child_nodes) {
                            Ok(data) => data,
                            Err(error) => { dbg!(error); vec![]},
                        }
                    } else {
                        vec![]
                    };


                    let mut record_group = Vec::with_capacity(child.row_count() as usize);
                    for row in 0..child.row_count() {
                        let child = child.child_1a(row);
                        record_group.push(get_node_type_from_tree_node(Some(child), model));
                    }

                    // If we have record nodes, move their data into the parent node data.
                    if !record_group.is_empty() {
                        record_group.reverse();

                        for child_node in child_nodes.iter_mut() {
                            if let NodeType::Record(_) = child_node {
                                if let Some(record_node) = record_group.pop() {
                                    *child_node = record_node;
                                }
                            }
                        }
                    }

                    record_group_nodes.push(child_nodes);
                }

                // Save the children... of our node.
                node.set_children(record_group_nodes);
            }

            // No nested blocks means we can directly get the children.
            else {

                let child_nodes = item.data_1a(CHILD_NODES).to_string().to_std_string();
                let mut child_nodes: Vec<NodeType> = if !child_nodes.is_empty() {
                    match serde_json::from_str(&child_nodes) {
                        Ok(data) => data,
                        Err(error) => { dbg!(error); vec![]},
                    }
                } else {
                    vec![]
                };

                // Get the record nodes and their data from the TreeView.
                let record_count = item.row_count();
                let mut record_nodes = Vec::with_capacity(record_count as usize);
                for row in 0..record_count {
                    let child = item.child_1a(row);
                    record_nodes.push(get_node_type_from_tree_node(Some(child), model));
                }

                // If we have record nodes, move their data into the parent node data.
                if !record_nodes.is_empty() {
                    record_nodes.reverse();

                    for child_node in child_nodes.iter_mut() {
                        if let NodeType::Record(_) = child_node {
                            if let Some(record_node) = record_nodes.pop() {
                                *child_node = record_node;
                            }
                        }
                    }
                }

                // Save the children... of our node.
                let children = vec![child_nodes];
                node.set_children(children);
            }
        },

        // Only record nodes are allowed to be nodes on the TreeView.
        _ => panic!()
    }
    node
}
