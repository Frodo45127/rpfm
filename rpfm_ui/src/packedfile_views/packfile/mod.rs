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

use qt_widgets::q_abstract_item_view::SelectionMode;
use qt_widgets::QAction;
use qt_widgets::QGridLayout;
use qt_widgets::QLineEdit;
use qt_widgets::QPushButton;
use qt_widgets::QTreeView;

use qt_gui::QStandardItemModel;

use qt_core::CaseSensitivity;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;

use cpp_core::MutPtr;

use std::path::PathBuf;
use std::sync::atomic::AtomicPtr;

use rpfm_error::Result;

use crate::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::{new_treeview_filter_safe, trigger_treeview_filter_safe};
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::qtr;
use crate::packedfile_views::{PackedFileView, TheOneSlot, View};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::pack_tree::{PackTree, TreeViewOperation};
use crate::utils::atomic_from_mut_ptr;
use crate::utils::mut_ptr_from_atomic;

use self::slots::PackFileExtraViewSlots;

mod connections;
mod shortcuts;
pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of the extra PackFile.
pub struct PackFileExtraView {
    tree_view: AtomicPtr<QTreeView>,
    tree_model_filter: AtomicPtr<QSortFilterProxyModel>,
    tree_model: AtomicPtr<QStandardItemModel>,

    filter_line_edit: AtomicPtr<QLineEdit>,
    filter_autoexpand_matches_button: AtomicPtr<QPushButton>,
    filter_case_sensitive_button: AtomicPtr<QPushButton>,

    expand_all: AtomicPtr<QAction>,
    collapse_all: AtomicPtr<QAction>,
}

/// This struct contains the raw version of each pointer in `PackFileExtraView`, to be used when building the slots.
///
/// This is kinda a hack, because AtomicPtr cannot be copied, and we need a copy of the entire set of pointers available
/// for the construction of the slots. So we build this one, copy it for the slots, then move it into the `PackFileExtraView`.
#[derive(Clone, Copy)]
pub struct PackFileExtraViewRaw {
    tree_view: MutPtr<QTreeView>,
    tree_model_filter: MutPtr<QSortFilterProxyModel>,
    tree_model: MutPtr<QStandardItemModel>,

    filter_line_edit: MutPtr<QLineEdit>,
    filter_autoexpand_matches_button: MutPtr<QPushButton>,
    filter_case_sensitive_button: MutPtr<QPushButton>,

    expand_all: MutPtr<QAction>,
    collapse_all: MutPtr<QAction>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackFileExtraView`.
impl PackFileExtraView {

    /// This function creates a new PackedFileView, and sets up his slots and connections.
    pub unsafe fn new_view(
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
        let mut tree_view = QTreeView::new_0a().into_ptr();
        let tree_model = QStandardItemModel::new_0a().into_ptr();
        let mut tree_model_filter = new_treeview_filter_safe(&mut pack_file_view.get_mut_widget());
        tree_model_filter.set_source_model(tree_model);
        tree_view.set_model(tree_model_filter);
        tree_view.set_header_hidden(true);
        tree_view.set_animated(true);
        tree_view.set_uniform_row_heights(true);
        tree_view.set_selection_mode(SelectionMode::ExtendedSelection);
        tree_view.set_expands_on_double_click(false);
        //tree_view.set_context_menu_policy(ContextMenuPolicy::Custom);
        tree_view.update_treeview(true, TreeViewOperation::Build(true));

        // Create and configure the widgets to control the `TreeView`s filter.
        let mut filter_line_edit = QLineEdit::new();
        let mut filter_autoexpand_matches_button = QPushButton::from_q_string(&qtr("treeview_autoexpand"));
        let mut filter_case_sensitive_button = QPushButton::from_q_string(&qtr("treeview_aai"));
        filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_autoexpand_matches_button.set_checkable(true);
        filter_case_sensitive_button.set_checkable(true);

        // Create the extra actions for the TreeView.
        let expand_all = QAction::from_q_string(&qtr("treeview_expand_all")).into_ptr();
        let collapse_all = QAction::from_q_string(&qtr("treeview_collapse_all")).into_ptr();
        tree_view.add_action(expand_all);
        tree_view.add_action(collapse_all);

        // Add everything to the main widget's Layout.
        let mut layout: MutPtr<QGridLayout> = pack_file_view.get_mut_widget().layout().static_downcast_mut();
        layout.add_widget_5a(tree_view, 0, 0, 1, 3);
        layout.add_widget_5a(&mut filter_line_edit, 1, 0, 1, 1);
        layout.add_widget_5a(&mut filter_autoexpand_matches_button, 1, 1, 1, 1);
        layout.add_widget_5a(&mut filter_case_sensitive_button, 1, 2, 1, 1);

        // Build the slots and set up the shortcuts/connections/tip.
        let raw = PackFileExtraViewRaw{
            tree_view,
            tree_model_filter,
            tree_model: tree_model,

            filter_line_edit: filter_line_edit.into_ptr(),
            filter_autoexpand_matches_button: filter_autoexpand_matches_button.into_ptr(),
            filter_case_sensitive_button: filter_case_sensitive_button.into_ptr(),

            expand_all: expand_all,
            collapse_all: collapse_all,
        };

        let slots = PackFileExtraViewSlots::new(*app_ui, *pack_file_contents_ui, *global_search_ui, raw);
        let mut view = Self {
            tree_view: atomic_from_mut_ptr(raw.tree_view),
            tree_model_filter: atomic_from_mut_ptr(raw.tree_model_filter),
            tree_model: atomic_from_mut_ptr(raw.tree_model),

            filter_line_edit: atomic_from_mut_ptr(raw.filter_line_edit),
            filter_autoexpand_matches_button: atomic_from_mut_ptr(raw.filter_autoexpand_matches_button),
            filter_case_sensitive_button: atomic_from_mut_ptr(raw.filter_case_sensitive_button),

            expand_all: atomic_from_mut_ptr(raw.expand_all),
            collapse_all: atomic_from_mut_ptr(raw.collapse_all),
        };

        connections::set_connections(&view, &slots);
        shortcuts::set_shortcuts(&mut view);
        pack_file_view.view = View::PackFile(view);

        // Return success.
        Ok(TheOneSlot::PackFile(slots))
    }

    /// This function returns a mutable reference to the `TreeView` widget.
    pub fn get_mut_ptr_tree_view(&self) -> MutPtr<QTreeView> {
        mut_ptr_from_atomic(&self.tree_view)
    }

    /// This function returns a mutable reference to the `SortFilterProxyModel` widget.
    pub fn get_mut_ptr_tree_model_filter(&self) -> MutPtr<QSortFilterProxyModel> {
        mut_ptr_from_atomic(&self.tree_model_filter)
    }

    /// This function returns a mutable reference to the `Expand All` Action.
    pub fn get_mut_ptr_expand_all(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.expand_all)
    }

    /// This function returns a mutable reference to the `Collapse All` Action.
    pub fn get_mut_ptr_collapse_all(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.collapse_all)
    }

    /// This function returns a mutable reference to the `Filter` Line Edit.
    pub fn get_mut_ptr_filter_line_edit(&self) -> MutPtr<QLineEdit> {
        mut_ptr_from_atomic(&self.filter_line_edit)
    }

    /// This function returns a mutable reference to the `Autoexpand Matches` Button.
    pub fn get_mut_ptr_autoexpand_matches_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.filter_autoexpand_matches_button)
    }

    /// This function returns a mutable reference to the `Case Sensitive` Button.
    pub fn get_mut_ptr_case_sensitive_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.filter_case_sensitive_button)
    }

    // Function to filter the contents of the TreeView.
    pub unsafe fn filter_files(view: &PackFileExtraViewRaw) {

        // Set the pattern to search.
        let mut pattern = QRegExp::new_1a(&view.get_mut_ptr_filter_line_edit().text());

        // Check if the filter should be "Case Sensitive".
        let case_sensitive = view.get_mut_ptr_case_sensitive_button().is_checked();
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::CaseSensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::CaseInsensitive); }

        // Filter whatever it's in that column by the text we got.
        trigger_treeview_filter_safe(&mut view.get_mut_ptr_tree_model_filter(), &mut pattern);

        // Expand all the matches, if the option for it is enabled.
        if view.get_mut_ptr_autoexpand_matches_button().is_checked() {
            view.get_mut_ptr_tree_view().expand_all();
        }
    }
}

/// Implementation of `PackFileExtraViewRaw`.
impl PackFileExtraViewRaw {

    /// This function returns a mutable reference to the `TreeView` widget.
    pub fn get_mut_ptr_tree_view(&self) -> MutPtr<QTreeView> {
        self.tree_view
    }

    /// This function returns a pointer to the `SortFilterProxyModel` widget.
    pub fn get_mut_ptr_tree_model_filter(&self) -> MutPtr<QSortFilterProxyModel> {
        self.tree_model_filter
    }

    /// This function returns a mutable reference to the `Expand All` Action.
    pub fn get_mut_ptr_expand_all(&self) -> MutPtr<QAction> {
        self.expand_all
    }

    /// This function returns a mutable reference to the `Collapse All` Action.
    pub fn get_mut_ptr_collapse_all(&self) -> MutPtr<QAction> {
        self.collapse_all
    }

    /// This function returns a mutable reference to the `Filter` Line Edit.
    pub fn get_mut_ptr_filter_line_edit(&self) -> MutPtr<QLineEdit> {
        self.filter_line_edit
    }

    /// This function returns a mutable reference to the `Autoexpand Matches` Button.
    pub fn get_mut_ptr_autoexpand_matches_button(&self) -> MutPtr<QPushButton> {
        self.filter_autoexpand_matches_button
    }

    /// This function returns a mutable reference to the `Case Sensitive` Button.
    pub fn get_mut_ptr_case_sensitive_button(&self) -> MutPtr<QPushButton> {
        self.filter_case_sensitive_button
    }
}
