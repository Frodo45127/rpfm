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
Module with all the code for the ESFTree, the tree used for the ESF Views.

It's similar to the PackTree, but modified for the requeriments of the ESF files.
!*/

use qt_widgets::QTreeView;
use qt_widgets::q_header_view::ResizeMode;

use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;
use qt_gui::QListOfQStandardItem;

use qt_core::QBox;
use qt_core::ItemFlag;
use qt_core::QFlags;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QPtr;

use cpp_core::CppBox;

use rpfm_lib::packedfile::esf::{ESF, NodeType};


//-------------------------------------------------------------------------------//
//                          Enums & Structs (and trait)
//-------------------------------------------------------------------------------//

/// This trait adds multiple util functions to the `TreeView` you implement it for.
///
/// Keep in mind that this trait has been created with `ESF TreeView's` in mind, so his methods
/// may not be suitable for all purposes.
pub(crate) trait ESFTree {

    /// This function takes care of EVERY operation that manipulates the provided TreeView.
    /// It does one thing or another, depending on the operation we provide it.
    unsafe fn update_treeview(&self, has_filter: bool, operation: ESFTreeViewOperation);
}

/// This enum has the different possible operations we can do in a `TreeView`.
#[derive(Clone, Debug)]
pub enum ESFTreeViewOperation {

    /// Build the entire `TreeView` from the provided ESF data.
    Build(ESF),
}

//-------------------------------------------------------------------------------//
//                      Implementations of `ESFTree`
//-------------------------------------------------------------------------------//

/// Implementation of `ESFTree` for `QPtr<QTreeView>.
impl ESFTree for QBox<QTreeView> {

    unsafe fn update_treeview(&self, has_filter: bool, operation: ESFTreeViewOperation) {
        let filter: Option<QPtr<QSortFilterProxyModel>> = if has_filter { Some(self.model().static_downcast()) } else { None };
        let model: QPtr<QStandardItemModel> = if let Some(ref filter) = filter { filter.source_model().static_downcast() } else { self.model().static_downcast() };

        // We act depending on the operation requested.
        match operation {

            // If we want to build a new TreeView...
            ESFTreeViewOperation::Build(packed_file_data) => {

                // First, we clean the TreeStore and whatever was created in the TreeView.
                model.clear();

                // Second, we set as the big_parent, the base for the folders of the TreeView, a fake folder
                // with the name of the PackFile. All big things start with a lie.
                let root_node = packed_file_data.get_ref_root_node();
                match root_node {
                    NodeType::Record(node) => {

                        let big_parent = QStandardItem::from_q_string(&QString::from_std_str(&node.get_ref_name()));
                        let state_item = QStandardItem::new();
                        big_parent.set_editable(false);
                        state_item.set_editable(false);
                        let flags = ItemFlag::from(state_item.flags().to_int() & ItemFlag::ItemIsSelectable.to_int());
                        state_item.set_flags(QFlags::from(flags));

                        load_node_to_view(&big_parent, node.get_ref_children());

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
                    _ => {}
                }
            },
        }
    }
}

unsafe fn load_node_to_view(parent: &CppBox<QStandardItem>, children: &[NodeType]) {
    for child in children {

        match child {
            NodeType::Record(node) => {
                let child_node = QStandardItem::from_q_string(&QString::from_std_str(node.get_ref_name()));
                let state_item = QStandardItem::new();

                load_node_to_view(&child_node, node.get_ref_children());

                let qlist = QListOfQStandardItem::new();
                qlist.append_q_standard_item(&child_node.into_ptr().as_mut_raw_ptr());
                qlist.append_q_standard_item(&state_item.into_ptr().as_mut_raw_ptr());


                parent.append_row_q_list_of_q_standard_item(qlist.as_ref());
            }

            NodeType::RecordBlock(node) => {
                let child_node = QStandardItem::from_q_string(&QString::from_std_str(node.get_ref_name()));
                let state_item = QStandardItem::new();

                for child in node.get_ref_children() {
                    load_node_to_view(&child_node, &child.1);
                }

                let qlist = QListOfQStandardItem::new();
                qlist.append_q_standard_item(&child_node.into_ptr().as_mut_raw_ptr());
                qlist.append_q_standard_item(&state_item.into_ptr().as_mut_raw_ptr());


                parent.append_row_q_list_of_q_standard_item(qlist.as_ref());
            }

            _ => {}
        }
    }
}
