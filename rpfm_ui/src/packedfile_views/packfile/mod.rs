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
Module with all the code for managing the temporal PackFile TreeView used when adding PackedFiles from another PackFile.

This is here because we're going to treat it as another PackedFileView, though it isn't.
But this allow us to integrate it into the main PackedFileView system, so it's ok.
!*/

use qt_widgets::QAction;
use qt_widgets::QGridLayout;
use qt_widgets::QLineEdit;
use qt_widgets::QMenu;
use qt_widgets::QToolButton;
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

use anyhow::Result;
use getset::Getters;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::{new_treeview_filter_safe, trigger_treeview_filter_safe};
use crate::locale::qtr;
use crate::packedfile_views::{BuildData, DataSource, PackedFileView, View, ViewType};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::pack_tree::{PackTree, TreeViewOperation};
use crate::utils::*;

use self::slots::PackFileExtraViewSlots;

mod connections;
pub mod slots;

/// Tool's ui template path.
const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/filterable_tree_widget.ui";
const VIEW_RELEASE: &str = "ui/filterable_tree_widget.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of the extra PackFile.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct PackFileExtraView {
    pack_file_path: Arc<RwLock<PathBuf>>,
    tree_view: QPtr<QTreeView>,
    tree_model_filter: QBox<QSortFilterProxyModel>,

    filter_line_edit: QPtr<QLineEdit>,
    filter_autoexpand_matches_button: QPtr<QToolButton>,
    filter_case_sensitive_button: QPtr<QToolButton>,

    context_menu: QBox<QMenu>,
    expand: QPtr<QAction>,
    collapse: QPtr<QAction>,
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
        let receiver = CENTRAL_COMMAND.send_background(Command::OpenPackExtra(pack_file_path.clone()));
        let response = CentralCommand::recv(&receiver);
        match response {
            Response::ContainerInfo(_) => {},
            Response::Error(error) => return Err(error),
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        }

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(pack_file_view.get_mut_widget(), &template_path)?;

        // Add everything to the main widget's Layout.
        let layout: QPtr<QGridLayout> = pack_file_view.get_mut_widget().layout().static_downcast();
        layout.add_widget_5a(&main_widget, 0, 0, 1, 1);

        let tree_view: QPtr<QTreeView> = find_widget(&main_widget.static_upcast(), "tree_view")?;
        let filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "filter_line_edit")?;
        let filter_autoexpand_matches_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_autoexpand_matches_button")?;
        let filter_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_case_sensitive_button")?;

        // Create and configure the `TreeView` itself.
        let tree_model = QStandardItemModel::new_1a(pack_file_view.get_mut_widget());
        let tree_model_filter = new_treeview_filter_safe(pack_file_view.get_mut_widget().static_upcast());
        tree_model_filter.set_source_model(&tree_model);
        tree_view.set_model(&tree_model_filter);
        tree_view.set_expands_on_double_click(false);
        filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));

        let mut build_data = BuildData::new();
        build_data.path = Some(pack_file_path.clone());
        build_data.editable = false;
        tree_view.update_treeview(true, TreeViewOperation::Build(build_data), DataSource::PackFile);

        // Create the extra actions for the TreeView.
        let context_menu = QMenu::from_q_widget(pack_file_view.get_mut_widget());
        let expand = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "secondary_pack_tree_context_menu", "expand", "treeview_expand_all", Some(pack_file_view.get_mut_widget().static_upcast::<qt_widgets::QWidget>()));
        let collapse = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "secondary_pack_tree_context_menu", "collapse", "treeview_collapse_all", Some(pack_file_view.get_mut_widget().static_upcast::<qt_widgets::QWidget>()));

        // Build the slots and set up the shortcuts/connections/tip.
        let view = Arc::new(PackFileExtraView{
            pack_file_path: Arc::new(RwLock::new(pack_file_path)),
            tree_view,
            tree_model_filter,

            filter_line_edit,
            filter_autoexpand_matches_button,
            filter_case_sensitive_button,

            context_menu,
            expand,
            collapse,
        });

        let slots = PackFileExtraViewSlots::new(app_ui, pack_file_contents_ui, &view);

        connections::set_connections(&view, &slots);
        pack_file_view.view = ViewType::Internal(View::PackFile(view));

        // Return success.
        Ok(())
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
