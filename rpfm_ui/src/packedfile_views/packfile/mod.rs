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
Module with all the code for managing the temporal PackFile TreeView used when adding PackedFiles from another PackFile.

This is here because we're going to treat it as another PackedFileView, though it isn't.
But this allow us to integrate it into the main PackedFileView system, so it's ok.
!*/

use qt_widgets::abstract_item_view::SelectionMode;
use qt_widgets::action::Action;
use qt_widgets::grid_layout::GridLayout;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::push_button::PushButton;
use qt_widgets::tree_view::TreeView;
use qt_widgets::widget::Widget;

use qt_gui::standard_item_model::StandardItemModel;

use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::object::Object;
use qt_core::qt::CaseSensitivity;
use qt_core::reg_exp::RegExp;
use qt_core::sort_filter_proxy_model::SortFilterProxyModel;

use std::path::PathBuf;
use std::sync::atomic::{AtomicPtr, Ordering};

use rpfm_error::Result;

use crate::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::{new_treeview_filter, trigger_treeview_filter};
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::qtr;
use crate::packedfile_views::{PackedFileView, TheOneSlot, View};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::pack_tree::{PackTree, TreeViewOperation};

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
    tree_model_filter: AtomicPtr<SortFilterProxyModel>,
    tree_model: AtomicPtr<StandardItemModel>,

    filter_line_edit: AtomicPtr<LineEdit>,
    filter_autoexpand_matches_button: AtomicPtr<PushButton>,
    filter_case_sensitive_button: AtomicPtr<PushButton>,

    expand_all: AtomicPtr<Action>,
    collapse_all: AtomicPtr<Action>,
}

/// This struct contains the raw version of each pointer in `PackFileExtraView`, to be used when building the slots.
///
/// This is kinda a hack, because AtomicPtr cannot be copied, and we need a copy of the entire set of pointers available
/// for the construction of the slots. So we build this one, copy it for the slots, then move it into the `PackFileExtraView`.
#[derive(Clone, Copy)]
pub struct PackFileExtraViewRaw {
    tree_view: *mut TreeView,
    tree_model_filter: *mut SortFilterProxyModel,
    tree_model: *mut StandardItemModel,

    filter_line_edit: *mut LineEdit,
    filter_autoexpand_matches_button: *mut PushButton,
    filter_case_sensitive_button: *mut PushButton,

    expand_all: *mut Action,
    collapse_all: *mut Action,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackFileExtraView`.
impl PackFileExtraView {

    /// This function creates a new PackedFileView, and sets up his slots and connections.
    pub fn new_view(
        pack_file_view: &mut PackedFileView,
        app_ui: &AppUI,
        pack_file_contents_ui: &PackFileContentsUI,
        global_search_ui: &GlobalSearchUI,
        pack_file_path: PathBuf,
    ) -> Result<TheOneSlot> {

        // Load the extra PackFile to memory.
        // Ignore the response, we don't need it yet.
        // TODO: Use this data to populate tooltips.
        CENTRAL_COMMAND.send_message_qt(Command::OpenPackFileExtra(pack_file_path));
        let response = CENTRAL_COMMAND.recv_message_qt();
        match response {
            Response::PackFileInfo(_) => {},
            Response::Error(error) => return Err(error),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        }

        // Create and configure the `TreeView` itself.
        let mut tree_view = TreeView::new();
        let tree_model = StandardItemModel::new(());
        let tree_model_filter = unsafe { new_treeview_filter(pack_file_view.get_mut_widget() as *mut Object) };
        unsafe { tree_model_filter.as_mut().unwrap().set_source_model(tree_model.as_mut_ptr() as *mut AbstractItemModel); }
        unsafe { tree_view.set_model(tree_model_filter as *mut AbstractItemModel); }
        tree_view.set_header_hidden(true);
        tree_view.set_animated(true);
        tree_view.set_uniform_row_heights(true);
        tree_view.set_selection_mode(SelectionMode::Extended);
        tree_view.set_expands_on_double_click(false);
        //tree_view.set_context_menu_policy(ContextMenuPolicy::Custom);
        tree_view.as_mut_ptr().update_treeview(true, TreeViewOperation::Build(true));

        // Create and configure the widgets to control the `TreeView`s filter.
        let mut filter_line_edit = LineEdit::new(());
        let mut filter_autoexpand_matches_button = PushButton::new(&qtr("treeview_autoexpand"));
        let mut filter_case_sensitive_button = PushButton::new(&qtr("treeview_aai"));
        filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_autoexpand_matches_button.set_checkable(true);
        filter_case_sensitive_button.set_checkable(true);

        // Create the extra actions for the TreeView.
        let expand_all = Action::new(&qtr("treeview_expand_all"));
        let collapse_all = Action::new(&qtr("treeview_collapse_all"));
        unsafe { tree_view.add_action(expand_all.as_mut_ptr()); }
        unsafe { tree_view.add_action(collapse_all.as_mut_ptr()); }

        // Add everything to the main widget's Layout.
        let layout = unsafe { pack_file_view.get_mut_widget().as_mut().unwrap().layout() as *mut GridLayout };
        unsafe { layout.as_mut().unwrap().add_widget((tree_view.as_mut_ptr() as *mut Widget, 0, 0, 1, 3)); }
        unsafe { layout.as_mut().unwrap().add_widget((filter_line_edit.as_mut_ptr() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { layout.as_mut().unwrap().add_widget((filter_autoexpand_matches_button.as_mut_ptr() as *mut Widget, 1, 1, 1, 1)); }
        unsafe { layout.as_mut().unwrap().add_widget((filter_case_sensitive_button.as_mut_ptr() as *mut Widget, 1, 2, 1, 1)); }

        // Build the slots and set up the shortcuts/connections/tip.
        let raw = PackFileExtraViewRaw{
            tree_view: tree_view.into_raw(),
            tree_model_filter,
            tree_model: tree_model.into_raw(),

            filter_line_edit: filter_line_edit.into_raw(),
            filter_autoexpand_matches_button: filter_autoexpand_matches_button.into_raw(),
            filter_case_sensitive_button: filter_case_sensitive_button.into_raw(),

            expand_all: expand_all.into_raw(),
            collapse_all: collapse_all.into_raw(),
        };

        let slots = PackFileExtraViewSlots::new(*app_ui, *pack_file_contents_ui, *global_search_ui, raw);
        let view = Self {
            tree_view: AtomicPtr::new(raw.tree_view),
            tree_model_filter: AtomicPtr::new(raw.tree_model_filter),
            tree_model: AtomicPtr::new(raw.tree_model),

            filter_line_edit: AtomicPtr::new(raw.filter_line_edit),
            filter_autoexpand_matches_button: AtomicPtr::new(raw.filter_autoexpand_matches_button),
            filter_case_sensitive_button: AtomicPtr::new(raw.filter_case_sensitive_button),

            expand_all: AtomicPtr::new(raw.expand_all),
            collapse_all: AtomicPtr::new(raw.collapse_all),
        };

        connections::set_connections(&view, &slots);
        shortcuts::set_shortcuts(&view);
        pack_file_view.view = View::PackFile(view);

        // Return success.
        Ok(TheOneSlot::PackFile(slots))
    }

    /// This function returns a mutable reference to the `TreeView` widget.
    pub fn get_ref_mut_tree_view(&self) -> &mut TreeView {
        unsafe { self.tree_view.load(Ordering::SeqCst).as_mut().unwrap() }
    }

    /// This function returns a mutable reference to the `SortFilterProxyModel` widget.
    pub fn get_ref_mut_tree_model_filter(&self) -> &mut SortFilterProxyModel {
        unsafe { self.tree_model_filter.load(Ordering::SeqCst).as_mut().unwrap() }
    }

    /// This function returns a pointer to the `SortFilterProxyModel` widget.
    pub fn get_tree_model_filter(&self) -> *mut SortFilterProxyModel {
        self.tree_model_filter.load(Ordering::SeqCst)
    }

    /// This function returns a mutable reference to the `Expand All` Action.
    pub fn get_ref_mut_expand_all(&self) -> &mut Action {
        unsafe { self.expand_all.load(Ordering::SeqCst).as_mut().unwrap() }
    }

    /// This function returns a mutable reference to the `Collapse All` Action.
    pub fn get_ref_mut_collapse_all(&self) -> &mut Action {
        unsafe { self.collapse_all.load(Ordering::SeqCst).as_mut().unwrap() }
    }

    /// This function returns a mutable reference to the `Filter` Line Edit.
    pub fn get_ref_mut_filter_line_edit(&self) -> &mut LineEdit {
        unsafe { self.filter_line_edit.load(Ordering::SeqCst).as_mut().unwrap() }
    }

    /// This function returns a mutable reference to the `Autoexpand Matches` Button.
    pub fn get_ref_mut_autoexpand_matches_button(&self) -> &mut PushButton {
        unsafe { self.filter_autoexpand_matches_button.load(Ordering::SeqCst).as_mut().unwrap() }
    }

    /// This function returns a mutable reference to the `Case Sensitive` Button.
    pub fn get_ref_mut_case_sensitive_button(&self) -> &mut PushButton {
        unsafe { self.filter_case_sensitive_button.load(Ordering::SeqCst).as_mut().unwrap() }
    }

    // Function to filter the contents of the TreeView.
    pub fn filter_files(view: &PackFileExtraViewRaw) {

        // Set the pattern to search.
        let mut pattern = RegExp::new(&view.get_ref_mut_filter_line_edit().text());

        // Check if the filter should be "Case Sensitive".
        let case_sensitive = view.get_ref_mut_case_sensitive_button().is_checked();
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::Sensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::Insensitive); }

        // Filter whatever it's in that column by the text we got.
        unsafe { trigger_treeview_filter(view.get_tree_model_filter(), &mut pattern); }

        // Expand all the matches, if the option for it is enabled.
        if view.get_ref_mut_autoexpand_matches_button().is_checked() {
            view.get_ref_mut_tree_view().expand_all();
        }
    }
}

/// Implementation of `PackFileExtraViewRaw`.
impl PackFileExtraViewRaw {

    /// This function returns a mutable reference to the `TreeView` widget.
    pub fn get_ref_mut_tree_view(&self) -> &mut TreeView {
        unsafe { self.tree_view.as_mut().unwrap() }
    }

    /// This function returns a mutable reference to the `SortFilterProxyModel` widget.
    pub fn get_ref_mut_tree_model_filter(&self) -> &mut SortFilterProxyModel {
        unsafe { self.tree_model_filter.as_mut().unwrap() }
    }

    /// This function returns a pointer to the `SortFilterProxyModel` widget.
    pub fn get_tree_model_filter(&self) -> *mut SortFilterProxyModel {
        self.tree_model_filter
    }

    /// This function returns a mutable reference to the `Expand All` Action.
    pub fn get_ref_mut_expand_all(&self) -> &mut Action {
        unsafe { self.expand_all.as_mut().unwrap() }
    }

    /// This function returns a mutable reference to the `Collapse All` Action.
    pub fn get_ref_mut_collapse_all(&self) -> &mut Action {
        unsafe { self.collapse_all.as_mut().unwrap() }
    }

    /// This function returns a mutable reference to the `Filter` Line Edit.
    pub fn get_ref_mut_filter_line_edit(&self) -> &mut LineEdit {
        unsafe { self.filter_line_edit.as_mut().unwrap() }
    }

    /// This function returns a mutable reference to the `Autoexpand Matches` Button.
    pub fn get_ref_mut_autoexpand_matches_button(&self) -> &mut PushButton {
        unsafe { self.filter_autoexpand_matches_button.as_mut().unwrap() }
    }

    /// This function returns a mutable reference to the `Case Sensitive` Button.
    pub fn get_ref_mut_case_sensitive_button(&self) -> &mut PushButton {
        unsafe { self.filter_case_sensitive_button.as_mut().unwrap() }
    }
}
