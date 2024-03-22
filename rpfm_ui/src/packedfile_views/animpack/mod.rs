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
Module with all the code for managing the view for AnimPack PackedFiles.
!*/

use qt_widgets::QAction;
use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QToolButton;
use qt_widgets::QTreeView;

use qt_gui::QStandardItemModel;

use qt_core::CaseSensitivity;
use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;

use anyhow::Result;
use getset::*;

use std::rc::Rc;
use std::sync::{Arc, RwLock};

use rpfm_lib::files::FileType;

use rpfm_ui_common::locale::qtr;

use crate::app_ui::AppUI;
use crate::backend::RFileInfo;
use crate::communications::*;
use crate::ffi::*;
use crate::pack_tree::PackTree;
use crate::packedfile_views::{BuildData, DataSource, FileView, TreeViewOperation, View, ViewType};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::*;

use self::slots::PackedFileAnimPackViewSlots;

mod connections;
mod slots;

/// Tool's ui template path.
const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/filterable_tree_widget.ui";
const VIEW_RELEASE: &str = "ui/filterable_tree_widget.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of an AnimPack PackedFile.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct PackedFileAnimPackView {
    path: Arc<RwLock<String>>,
    #[getset(skip)]
    data_source: Arc<RwLock<DataSource>>,

    pack_tree_view: QPtr<QTreeView>,
    pack_tree_model_filter: QBox<QSortFilterProxyModel>,

    pack_filter_line_edit: QPtr<QLineEdit>,
    pack_filter_autoexpand_matches_button: QPtr<QToolButton>,
    pack_filter_case_sensitive_button: QPtr<QToolButton>,

    pack_expand_all: QPtr<QAction>,
    pack_collapse_all: QPtr<QAction>,

    anim_pack_tree_view: QPtr<QTreeView>,
    anim_pack_tree_model_filter: QBox<QSortFilterProxyModel>,
    _anim_pack_tree_model: QBox<QStandardItemModel>,

    anim_pack_filter_line_edit: QPtr<QLineEdit>,
    anim_pack_filter_autoexpand_matches_button: QPtr<QToolButton>,
    anim_pack_filter_case_sensitive_button: QPtr<QToolButton>,

    anim_pack_expand_all: QPtr<QAction>,
    anim_pack_collapse_all: QPtr<QAction>,
    anim_pack_delete: QPtr<QAction>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileAnimPackView`.
impl PackedFileAnimPackView {

    /// This function creates a new AnimPack View, and sets up his slots and connections.
    pub unsafe fn new_view(
        file_view: &mut FileView,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        file_info: &RFileInfo,
        files_info: &[RFileInfo],
    ) -> Result<()> {
        let layout: QPtr<QGridLayout> = file_view.main_widget().layout().static_downcast();

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget_left = load_template(file_view.main_widget(), template_path)?;
        let main_widget_right = load_template(file_view.main_widget(), template_path)?;

        let pack_tree_view: QPtr<QTreeView> = find_widget(&main_widget_left.static_upcast(), "tree_view")?;
        let pack_filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget_left.static_upcast(), "filter_line_edit")?;
        let pack_filter_autoexpand_matches_button: QPtr<QToolButton> = find_widget(&main_widget_left.static_upcast(), "filter_autoexpand_matches_button")?;
        let pack_filter_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget_left.static_upcast(), "filter_case_sensitive_button")?;

        let anim_pack_tree_view: QPtr<QTreeView> = find_widget(&main_widget_right.static_upcast(), "tree_view")?;
        let anim_pack_filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget_right.static_upcast(), "filter_line_edit")?;
        let anim_pack_filter_autoexpand_matches_button: QPtr<QToolButton> = find_widget(&main_widget_right.static_upcast(), "filter_autoexpand_matches_button")?;
        let anim_pack_filter_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget_right.static_upcast(), "filter_case_sensitive_button")?;

        // Create and configure the left `TreeView`, AKA the open PackFile.
        let instructions = QLabel::from_q_string_q_widget(&qtr("animpack_view_instructions"), file_view.main_widget());
        let tree_model = pack_file_contents_ui.packfile_contents_tree_model();
        let pack_tree_model_filter = new_treeview_filter_safe(file_view.main_widget().static_upcast());
        pack_tree_model_filter.set_source_model(tree_model);
        pack_tree_view.set_model(&pack_tree_model_filter);
        pack_tree_view.set_expands_on_double_click(false);
        pack_tree_view.header().set_stretch_last_section(true);

        // Create and configure the widgets to control the `TreeView`s filter.
        pack_filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));

        // Create the extra actions for the TreeView.
        let pack_expand_all = add_action_to_widget(app_ui.shortcuts().as_ref(), "anim_pack_tree_context_menu", "pack_expand_all", Some(pack_tree_view.static_upcast()));
        let pack_collapse_all = add_action_to_widget(app_ui.shortcuts().as_ref(), "anim_pack_tree_context_menu", "pack_collapse_all", Some(pack_tree_view.static_upcast()));

        // Add everything to the main widget's Layout.
        layout.add_widget_5a(&instructions, 0, 0, 1, 2);
        layout.add_widget_5a(&main_widget_left, 1, 0, 1, 1);
        layout.add_widget_5a(&main_widget_right, 1, 1, 1, 1);

        // Create and configure the right `TreeView`, AKA the AnimPack.
        let anim_pack_tree_model = QStandardItemModel::new_1a(file_view.main_widget());
        let anim_pack_tree_model_filter = new_treeview_filter_safe(file_view.main_widget().static_upcast());
        anim_pack_tree_model_filter.set_source_model(&anim_pack_tree_model);
        anim_pack_tree_view.set_model(&anim_pack_tree_model_filter);
        anim_pack_tree_view.set_expands_on_double_click(false);
        anim_pack_tree_view.header().set_stretch_last_section(true);

        let mut build_data = BuildData::new();
        let container_info = From::from(file_info);
        build_data.data = Some((container_info, files_info.to_vec()));
        build_data.editable = false;
        anim_pack_tree_view.update_treeview(true, TreeViewOperation::Build(build_data), DataSource::PackFile);

        // Create and configure the widgets to control the `TreeView`s filter.
        anim_pack_filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));

        // Create the extra actions for the TreeView.
        let anim_pack_expand_all = add_action_to_widget(app_ui.shortcuts().as_ref(), "anim_pack_tree_context_menu", "expand_all", Some(anim_pack_tree_view.static_upcast()));
        let anim_pack_collapse_all = add_action_to_widget(app_ui.shortcuts().as_ref(), "anim_pack_tree_context_menu", "collapse_all", Some(anim_pack_tree_view.static_upcast()));
        let anim_pack_delete = add_action_to_widget(app_ui.shortcuts().as_ref(), "anim_pack_tree_context_menu", "delete", Some(anim_pack_tree_view.static_upcast()));

        let packed_file_animpack_view = Arc::new(PackedFileAnimPackView {
            path: file_view.path_raw(),
            data_source: file_view.data_source.clone(),

            pack_tree_view,
            pack_tree_model_filter,

            pack_filter_line_edit,
            pack_filter_autoexpand_matches_button,
            pack_filter_case_sensitive_button,

            pack_expand_all,
            pack_collapse_all,

            anim_pack_tree_view,
            anim_pack_tree_model_filter,
            _anim_pack_tree_model: anim_pack_tree_model,

            anim_pack_filter_line_edit,
            anim_pack_filter_autoexpand_matches_button,
            anim_pack_filter_case_sensitive_button,

            anim_pack_expand_all,
            anim_pack_collapse_all,
            anim_pack_delete
        });

        let packed_file_animpack_view_slots = PackedFileAnimPackViewSlots::new(
            &packed_file_animpack_view,
            app_ui,
            pack_file_contents_ui
        );

        connections::set_connections(&packed_file_animpack_view, &packed_file_animpack_view_slots);
        file_view.view_type = ViewType::Internal(View::AnimPack(packed_file_animpack_view));
        file_view.file_type = FileType::AnimPack;

        Ok(())
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: (&RFileInfo, Vec<RFileInfo>)) {
        let mut build_data = BuildData::new();
        let container_info = From::from(data.0);
        build_data.data = Some((container_info, data.1));
        build_data.editable = false;
        self.anim_pack_tree_view.update_treeview(true, TreeViewOperation::Build(build_data), DataSource::PackFile);
    }

    /// Function to filter the TreeViews.
    pub unsafe fn filter_files(ui: &Arc<Self>, is_anim_pack: bool) {

        let tree_view = if is_anim_pack { &ui.anim_pack_tree_view } else { &ui.pack_tree_view };
        let tree_model_filter = if is_anim_pack { &ui.anim_pack_tree_model_filter } else { &ui.pack_tree_model_filter };
        let filter_line_edit = if is_anim_pack { &ui.anim_pack_filter_line_edit } else { &ui.pack_filter_line_edit };
        let filter_case_sensitive_button = if is_anim_pack { &ui.anim_pack_filter_case_sensitive_button } else { &ui.pack_filter_case_sensitive_button };
        let filter_autoexpand_matches_button = if is_anim_pack { &ui.anim_pack_filter_autoexpand_matches_button } else { &ui.pack_filter_autoexpand_matches_button };

        // Set the pattern to search.
        let pattern = QRegExp::new_1a(&filter_line_edit.text());

        // Check if the filter should be "Case Sensitive".
        let case_sensitive = filter_case_sensitive_button.is_checked();
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::CaseSensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::CaseInsensitive); }

        // Filter whatever it's in that column by the text we got.
        trigger_treeview_filter_safe(tree_model_filter, &pattern.as_ptr());

        // Expand all the matches, if the option for it is enabled.
        if filter_autoexpand_matches_button.is_checked() {
            tree_view.expand_all();
        }
    }
}
