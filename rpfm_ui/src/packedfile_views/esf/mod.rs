//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code for managing the ESF Views.
!*/

use qt_widgets::q_abstract_item_view::SelectionMode;
use qt_widgets::QLineEdit;
use qt_widgets::QPushButton;
use qt_widgets::QGridLayout;
use qt_widgets::QSplitter;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

use qt_gui::QStandardItemModel;

use qt_core::CaseSensitivity;
use qt_core::ContextMenuPolicy;
use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;
use qt_core::QTimer;

use std::rc::Rc;
use std::sync::{Arc, RwLock};

use anyhow::Result;

use rpfm_lib::files::{esf::ESF, FileType};

use crate::app_ui::AppUI;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::ffi::*;
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::qtr;
use crate::packedfile_views::esf::esftree::*;
use crate::packedfile_views::esf::slots::PackedFileESFViewSlots;
use crate::packedfile_views::PackedFileView;
use crate::packedfile_views::PackFileContentsUI;
use crate::references_ui::ReferencesUI;
use crate::utils::create_grid_layout;

use self::esf_detailed_view::ESFDetailedView;

use super::{ViewType, View};

mod connections;
mod esftree;
mod esf_detailed_view;
mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of the ESF PackedFile.
pub struct PackedFileESFView {
    tree_view: QBox<QTreeView>,
    _tree_model: QBox<QStandardItemModel>,
    tree_filter: QBox<QSortFilterProxyModel>,

    filter_line_edit: QBox<QLineEdit>,
    filter_autoexpand_matches_button: QBox<QPushButton>,
    filter_case_sensitive_button: QBox<QPushButton>,
    filter_timer_delayed_updates: QBox<QTimer>,

    node_data_panel: QBox<QWidget>,

    detailed_view: Arc<RwLock<ESFDetailedView>>,

    _path: Arc<RwLock<String>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileESFView`.
impl PackedFileESFView {

    /// This function creates a new PackedFileESFView, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_view: &mut PackedFileView,
        app_ui: &Rc<AppUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
        data: ESF
    ) -> Result<()> {

        let splitter = QSplitter::from_q_widget(packed_file_view.get_mut_widget());

        // Create the TreeView for the ESF PackedFile.
        let tree_view = QTreeView::new_1a(packed_file_view.get_mut_widget());
        let tree_model = new_packed_file_model_safe();
        let tree_filter = new_treeview_filter_safe(tree_view.static_upcast());
        tree_filter.set_source_model(&tree_model);
        tree_model.set_parent(&tree_view);
        tree_view.set_model(&tree_filter);
        tree_view.set_header_hidden(true);
        tree_view.set_animated(true);
        tree_view.set_uniform_row_heights(true);
        tree_view.set_selection_mode(SelectionMode::ExtendedSelection);
        tree_view.set_context_menu_policy(ContextMenuPolicy::CustomContextMenu);
        tree_view.set_expands_on_double_click(true);
        tree_view.header().set_stretch_last_section(false);

        // Create and configure the widgets to control the `TreeView`s filter.
        let filter_timer_delayed_updates = QTimer::new_1a(packed_file_view.get_mut_widget());
        let filter_line_edit = QLineEdit::from_q_widget(packed_file_view.get_mut_widget());
        let filter_autoexpand_matches_button = QPushButton::from_q_string_q_widget(&qtr("treeview_autoexpand"), packed_file_view.get_mut_widget());
        let filter_case_sensitive_button = QPushButton::from_q_string_q_widget(&qtr("treeview_aai"), packed_file_view.get_mut_widget());
        filter_timer_delayed_updates.set_single_shot(true);
        filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_line_edit.set_clear_button_enabled(true);
        filter_autoexpand_matches_button.set_checkable(true);
        filter_case_sensitive_button.set_checkable(true);

        let tree_panel = QWidget::new_1a(&splitter);
        let tree_layout = create_grid_layout(tree_panel.static_upcast());
        tree_panel.set_minimum_width(200);

        // Add everything to the `TreeView`s Layout.
        tree_layout.add_widget_5a(&tree_view, 0, 0, 1, 2);
        tree_layout.add_widget_5a(&filter_line_edit, 1, 0, 1, 2);
        tree_layout.add_widget_5a(&filter_autoexpand_matches_button, 2, 0, 1, 1);
        tree_layout.add_widget_5a(&filter_case_sensitive_button, 2, 1, 1, 1);

        let node_data_panel = QWidget::new_1a(&splitter);
        let node_data_layout = create_grid_layout(node_data_panel.static_upcast());
        node_data_layout.set_row_stretch(1000, 100);
        node_data_layout.set_column_stretch(1, 100);
        node_data_panel.set_minimum_width(250);

        let layout: QPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast();
        layout.add_widget_5a(&splitter, 0, 0, 1, 1);

        let view = Arc::new(Self {
            tree_view,
            _tree_model: tree_model,
            tree_filter,

            filter_line_edit,
            filter_autoexpand_matches_button,
            filter_case_sensitive_button,
            filter_timer_delayed_updates,

            node_data_panel,

            detailed_view: Arc::new(RwLock::new(ESFDetailedView::default())),

            _path: packed_file_view.get_path_raw()
        });

        view.tree_view.update_treeview(true, ESFTreeViewOperation::Build(data));

        let slots = PackedFileESFViewSlots::new(
            &view,
            app_ui,
            global_search_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui
        );

        connections::set_connections(&view, &slots);
        packed_file_view.view = ViewType::Internal(View::ESF(view));
        packed_file_view.packed_file_type = FileType::ESF;

        Ok(())
    }

    /// This function tries to reload the current view with the provided data.
    pub unsafe fn reload_view(&self, data: &ESF) {
        self.tree_view.update_treeview(true, ESFTreeViewOperation::Build(data.clone()));
    }

    /// This function saves the current view to an ESF struct.
    pub unsafe fn save_view(&self) -> ESF {

        // First, save the currently open node.
        self.detailed_view.read().unwrap().save_to_tree_node(&self.tree_view);

        // Then, generate an ESF struct from the tree data.
        self.tree_view.get_esf_from_view(true)
    }

    /// Function to filter the ESF TreeView.
    pub unsafe fn filter_files(view: &Arc<Self>) {

        // Set the pattern to search.
        let pattern = QRegExp::new_1a(&view.filter_line_edit.text());

        // Check if the filter should be "Case Sensitive".
        let case_sensitive = view.filter_case_sensitive_button.is_checked();
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::CaseSensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::CaseInsensitive); }

        // Filter whatever it's in that column by the text we got.
        trigger_treeview_filter_safe(&view.tree_filter, &pattern.as_ptr());

        // Expand all the matches, if the option for it is enabled.
        if view.filter_autoexpand_matches_button.is_checked() {
            view.tree_view.expand_all();
        }
    }

    pub unsafe fn start_delayed_updates_timer(view: &Arc<Self>,) {
        view.filter_timer_delayed_updates.set_interval(500);
        view.filter_timer_delayed_updates.start_0a();
    }
}
