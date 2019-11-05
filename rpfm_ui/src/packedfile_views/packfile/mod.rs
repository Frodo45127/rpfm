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
Module with all the code for managing the temporal PackFile TreeView used when adding PackedFiles from another PackFile.

This is here because we're going to treat it as another PackedFileView, though it isn't.
But this allow us to integrate it into the main PackedFileView system, so it's ok.
!*/

use qt_widgets::action::Action;
use qt_widgets::grid_layout::GridLayout;
use qt_widgets::tree_view::TreeView;
use qt_widgets::widget::Widget;

use qt_gui::standard_item_model::StandardItemModel;

use qt_core::abstract_item_model::AbstractItemModel;

use std::path::PathBuf;
use std::sync::atomic::{AtomicPtr, Ordering};

use rpfm_error::Result;

use crate::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::packedfile_views::{PackedFileView, TheOneSlot, View};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::pack_tree::{PackTree, TreeViewOperation};
use crate::QString;

use self::slots::PackFileExtraViewSlots;

mod connections;
mod shortcuts;
pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of the extra PackFile.
pub struct PackFileExtraView {
    tree_view: AtomicPtr<TreeView>,
    expand_all: AtomicPtr<Action>,
    collapse_all: AtomicPtr<Action>,
}

/// This struct contains the raw version of each pointer in `PackFileExtraView`, to be used when building the slots.
///
/// This is kinda a hack, because AtomicPtr cannot be copied, and we need a copy of the entire set of pointers available
/// for the construction of the slots. So we build this one, copy it for the slots, then move it into the `PackFileExtraView`.
#[derive(Clone, Copy)]
pub struct PackFileExtraViewRaw {
    pub tree_view: *mut TreeView,
    pub expand_all: *mut Action,
    pub collapse_all: *mut Action,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackFileExtraView`.
impl PackFileExtraView {

    /// This function creates a new PackedFileView, and sets up his slots and connections.
    pub fn new_view(
        packed_file_view: &mut PackedFileView,
        app_ui: &AppUI,
        pack_file_contents_ui: &PackFileContentsUI,
        pack_file_path: PathBuf,
    ) -> Result<TheOneSlot> {

        // Load the extra PackFile to memory.
        // Ignore the response, we don't need it yet.
        // TODO: Use this data to populate tooltips.
        CENTRAL_COMMAND.send_message_qt(Command::OpenPackFileExtra(pack_file_path));
        match CENTRAL_COMMAND.recv_message_qt() {
            Response::PackFileInfo(_) => {},
            Response::Error(error) => return Err(error),
            _ => panic!(THREADS_COMMUNICATION_ERROR),
        }

        // Create the TreeView and configure it.
        let mut tree_view = TreeView::new();
        let tree_model = StandardItemModel::new(());
        unsafe { tree_view.set_model(tree_model.into_raw() as *mut AbstractItemModel); }
        tree_view.set_header_hidden(true);
        tree_view.set_expands_on_double_click(false);
        tree_view.set_animated(true);
        tree_view.as_mut_ptr().update_treeview(false, TreeViewOperation::Build(true));

        // Create the extra actions for the TreeView.
        let expand_all = Action::new(&QString::from_std_str("&Expand All"));
        let collapse_all = Action::new(&QString::from_std_str("&Collapse All"));
        unsafe { tree_view.add_action(expand_all.as_mut_ptr()); }
        unsafe { tree_view.add_action(collapse_all.as_mut_ptr()); }

        // Add all the stuff to the Grid.
        let layout = unsafe { packed_file_view.get_mut_widget().as_mut().unwrap().layout() as *mut GridLayout };
        unsafe { layout.as_mut().unwrap().add_widget((tree_view.as_mut_ptr() as *mut Widget, 1, 0, 1, 1)); }

        // Build the slots and set up the shortcuts/connections/tip.
        let raw = PackFileExtraViewRaw{
            tree_view: tree_view.into_raw(),
            expand_all: expand_all.into_raw(),
            collapse_all: collapse_all.into_raw(),
        };

        let slots = PackFileExtraViewSlots::new(*app_ui, *pack_file_contents_ui, raw);
        let view = Self {
            tree_view: AtomicPtr::new(raw.tree_view),
            expand_all: AtomicPtr::new(raw.expand_all),
            collapse_all: AtomicPtr::new(raw.collapse_all),
        };

        connections::set_connections(&view, &slots);
        shortcuts::set_shortcuts(&view);
        packed_file_view.view = View::PackFile(view);

        // Return success.
        Ok(TheOneSlot::PackFile(slots))
    }

    /// This function returns a mutable reference to the `TreeView` widget.
    pub fn get_tree_view(&self) -> &mut TreeView {
        unsafe { self.tree_view.load(Ordering::SeqCst).as_mut().unwrap() }
    }

    /// This function returns a mutable reference to the `Expand All` Action.
    pub fn get_expand_all(&self) -> &mut Action {
        unsafe { self.expand_all.load(Ordering::SeqCst).as_mut().unwrap() }
    }

    /// This function returns a mutable reference to the `Collapse All` Action.
    pub fn get_collapse_all(&self) -> &mut Action {
        unsafe { self.collapse_all.load(Ordering::SeqCst).as_mut().unwrap() }
    }
}
