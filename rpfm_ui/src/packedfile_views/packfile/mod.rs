//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code for managing the temporal PackFile TreeView used when adding PackedFiles from another PackFile.

This is here because we're going to treat it as another FileView, though it isn't.
But this allow us to integrate it into the main FileView system, so it's ok.

The source Pack is opened on the backend as a temporary pack (its key is its path on disk) and shown
as a read-only tree. Importing a file copies it into the currently-selected open Pack. The temporary
source Pack is closed when this view's tab is closed (see `AppUI::purge_them` extra-packfile handling).
!*/

use qt_widgets::QGridLayout;
use qt_widgets::QLineEdit;
use qt_widgets::QMenu;
use qt_widgets::QToolButton;
use qt_widgets::QTreeView;

use qt_gui::QAction;
use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QFlags;
use qt_core::q_regular_expression;
use qt_core::QRegularExpression;

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Result};
use getset::Getters;

use rpfm_ipc::helpers::DataSource;

use rpfm_ui_common::utils::{find_widget, load_template};

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::{new_treeview_filter_safe, trigger_treeview_filter_safe};
use crate::packedfile_views::{FileView, View, ViewType};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::pack_tree::{BuildData, PackTree, TreeViewOperation};
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
    tree_model_filter: QBox<qt_core::QSortFilterProxyModel>,

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

    /// This function creates a new FileView, and sets up his slots and connections.
    pub unsafe fn new_view(
        pack_file_view: &mut FileView,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        pack_file_path: PathBuf,
    ) -> Result<()> {

        // Load the extra PackFile to memory via the unified packs map. Its key is its path on disk.
        let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::OpenPackFiles(vec![pack_file_path.clone()]));
        let response = CentralCommand::recv(&receiver);
        let pack_key = match response {
            Response::StringContainerInfo(key, _) => key,
            Response::Error(error) => return Err(anyhow!(error)),
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        };

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(pack_file_view.main_widget(), template_path)?;

        // Add everything to the main widget's Layout.
        let layout: QPtr<QGridLayout> = pack_file_view.main_widget().layout().static_downcast();
        layout.add_widget_5a(&main_widget, 0, 0, 1, 1);

        let tree_view: QPtr<QTreeView> = find_widget(&main_widget.static_upcast(), "tree_view")?;
        let filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "filter_line_edit")?;
        let filter_autoexpand_matches_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_autoexpand_matches_button")?;
        let filter_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_case_sensitive_button")?;

        // Create and configure the `TreeView` itself.
        let tree_model = QStandardItemModel::new_1a(pack_file_view.main_widget());
        let tree_model_filter = new_treeview_filter_safe(pack_file_view.main_widget().static_upcast());
        tree_model_filter.set_source_model(&tree_model);
        tree_view.set_model(&tree_model_filter);
        tree_view.set_expands_on_double_click(false);
        filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));

        // Build the source Pack's tree (read-only) by its key.
        let mut build_data = BuildData::new();
        build_data.editable = false;
        build_data.pack_key = Some(pack_key.clone());
        tree_view.update_treeview(true, TreeViewOperation::Build(build_data), DataSource::PackFile, &pack_key);

        // Create the extra actions for the TreeView.
        let context_menu = QMenu::from_q_widget(pack_file_view.main_widget());
        let expand = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "secondary_pack_tree_context_menu", "expand", "treeview_expand_all", Some(pack_file_view.main_widget().static_upcast::<qt_widgets::QWidget>()));
        let collapse = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "secondary_pack_tree_context_menu", "collapse", "treeview_collapse_all", Some(pack_file_view.main_widget().static_upcast::<qt_widgets::QWidget>()));

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
        pack_file_view.view_type = ViewType::Internal(View::PackFile(view));

        // Return success.
        Ok(())
    }

    /// Function to filter the contents of the TreeView.
    pub unsafe fn filter_files(view: &Arc<Self>) {

        // Set the pattern to search.
        let pattern = QRegularExpression::new_1a(&view.filter_line_edit.text());

        // Check if the filter should be "Case Sensitive".
        if !view.filter_case_sensitive_button.is_checked() {
            pattern.set_pattern_options(QFlags::from(q_regular_expression::PatternOption::CaseInsensitiveOption));
        }

        // Filter whatever it's in that column by the text we got.
        trigger_treeview_filter_safe(&view.tree_model_filter, &pattern.as_ptr());

        // Expand all the matches, if the option for it is enabled.
        if view.filter_autoexpand_matches_button.is_checked() {
            view.tree_view.expand_all();
        }
    }
}
