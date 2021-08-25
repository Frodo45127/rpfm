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

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::CaseSensitivity;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use rpfm_error::Result;
use rpfm_lib::packedfile::PackedFileType;

use crate::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::{new_treeview_filter_safe, trigger_treeview_filter_safe};
use crate::locale::qtr;
use crate::packedfile_views::{BuildData, DataSource, PackedFileView, View, ViewType};
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
    pack_file_path: Arc<RwLock<PathBuf>>,
    tree_view: QBox<QTreeView>,
    tree_model_filter: QBox<QSortFilterProxyModel>,

    filter_line_edit: QBox<QLineEdit>,
    filter_autoexpand_matches_button: QBox<QPushButton>,
    filter_case_sensitive_button: QBox<QPushButton>,

    expand_all: QBox<QAction>,
    collapse_all: QBox<QAction>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackFileExtraView`.
impl PackFileExtraView {

    /// This function creates a new PackedFileView, and sets up his slots and connections.
    pub unsafe fn new_view(
        pack_file_view: &mut PackedFileView,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        pack_file_path: PathBuf,
    ) -> Result<()> {

        // Load the extra PackFile to memory.
        // Ignore the response, we don't need it yet.
        // TODO: Use this data to populate tooltips.
        CENTRAL_COMMAND.send_message_qt(Command::OpenPackFileExtra(pack_file_path.clone()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        match response {
            Response::PackFileInfo(_) => {},
            Response::Error(error) => return Err(error),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        }

        // Create and configure the `TreeView` itself.
        let tree_view = QTreeView::new_1a(pack_file_view.get_mut_widget());
        let tree_model = QStandardItemModel::new_1a(pack_file_view.get_mut_widget());
        let tree_model_filter = new_treeview_filter_safe(pack_file_view.get_mut_widget().static_upcast());
        tree_model_filter.set_source_model(&tree_model);
        tree_view.set_model(&tree_model_filter);
        tree_view.set_header_hidden(true);
        tree_view.set_animated(true);
        tree_view.set_uniform_row_heights(true);
        tree_view.set_selection_mode(SelectionMode::ExtendedSelection);
        tree_view.set_expands_on_double_click(false);
        //tree_view.set_context_menu_policy(ContextMenuPolicy::Custom);
        //
        let mut build_data = BuildData::new();
        build_data.path = Some(pack_file_path.clone());
        build_data.editable = false;
        tree_view.update_treeview(true, TreeViewOperation::Build(build_data), DataSource::PackFile);

        // Create and configure the widgets to control the `TreeView`s filter.
        let filter_line_edit = QLineEdit::from_q_widget(pack_file_view.get_mut_widget());
        let filter_autoexpand_matches_button = QPushButton::from_q_string_q_widget(&qtr("treeview_autoexpand"), pack_file_view.get_mut_widget());
        let filter_case_sensitive_button = QPushButton::from_q_string_q_widget(&qtr("treeview_aai"), pack_file_view.get_mut_widget());
        filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_line_edit.set_clear_button_enabled(true);
        filter_autoexpand_matches_button.set_checkable(true);
        filter_case_sensitive_button.set_checkable(true);

        // Create the extra actions for the TreeView.
        let expand_all = QAction::from_q_string_q_object(&qtr("treeview_expand_all"), pack_file_view.get_mut_widget());
        let collapse_all = QAction::from_q_string_q_object(&qtr("treeview_collapse_all"), pack_file_view.get_mut_widget());
        tree_view.add_action(&expand_all);
        tree_view.add_action(&collapse_all);

        // Add everything to the main widget's Layout.
        let layout: QPtr<QGridLayout> = pack_file_view.get_mut_widget().layout().static_downcast();
        layout.add_widget_5a(&tree_view, 0, 0, 1, 3);
        layout.add_widget_5a(&filter_line_edit, 1, 0, 1, 1);
        layout.add_widget_5a(&filter_autoexpand_matches_button, 1, 1, 1, 1);
        layout.add_widget_5a(&filter_case_sensitive_button, 1, 2, 1, 1);

        // Build the slots and set up the shortcuts/connections/tip.
        let view = Arc::new(PackFileExtraView{
            pack_file_path: Arc::new(RwLock::new(pack_file_path)),
            tree_view,
            tree_model_filter,

            filter_line_edit,
            filter_autoexpand_matches_button,
            filter_case_sensitive_button,

            expand_all,
            collapse_all,
        });

        let slots = PackFileExtraViewSlots::new(app_ui, pack_file_contents_ui, &view);

        connections::set_connections(&view, &slots);
        shortcuts::set_shortcuts(&view);
        pack_file_view.packed_file_type = PackedFileType::PackFile;
        pack_file_view.view = ViewType::Internal(View::PackFile(view));

        // Return success.
        Ok(())
    }

    /// This function returns a mutable reference to the `TreeView` widget.
    pub fn get_mut_ptr_tree_view(&self) -> &QBox<QTreeView> {
        &self.tree_view
    }

    /// This function returns a mutable reference to the `Expand All` Action.
    pub fn get_mut_ptr_expand_all(&self) -> &QBox<QAction> {
        &self.expand_all
    }

    /// This function returns a mutable reference to the `Collapse All` Action.
    pub fn get_mut_ptr_collapse_all(&self) -> &QBox<QAction> {
        &self.collapse_all
    }

    /// This function returns a mutable reference to the `Filter` Line Edit.
    pub fn get_mut_ptr_filter_line_edit(&self) -> &QBox<QLineEdit> {
        &self.filter_line_edit
    }

    /// This function returns a mutable reference to the `Autoexpand Matches` Button.
    pub fn get_mut_ptr_autoexpand_matches_button(&self) -> &QBox<QPushButton> {
        &self.filter_autoexpand_matches_button
    }

    /// This function returns a mutable reference to the `Case Sensitive` Button.
    pub fn get_mut_ptr_case_sensitive_button(&self) -> &QBox<QPushButton> {
        &self.filter_case_sensitive_button
    }

    // Function to filter the contents of the TreeView.
    pub unsafe fn filter_files(view: &Arc<Self>) {

        // Set the pattern to search.
        let pattern = QRegExp::new_1a(&view.filter_line_edit.text());

        // Check if the filter should be "Case Sensitive".
        let case_sensitive = view.filter_case_sensitive_button.is_checked();
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::CaseSensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::CaseInsensitive); }

        // Filter whatever it's in that column by the text we got.
        trigger_treeview_filter_safe(&view.tree_model_filter, &pattern.as_ptr());

        // Expand all the matches, if the option for it is enabled.
        if view.filter_autoexpand_matches_button.is_checked() {
            view.tree_view.expand_all();
        }
    }
}
