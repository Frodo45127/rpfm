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
Module with all the code for managing the view for AnimPack PackedFiles.
!*/

use qt_widgets::q_abstract_item_view::SelectionMode;
use qt_widgets::QTreeView;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QAction;
use qt_widgets::QGridLayout;

use qt_widgets::QPushButton;

use qt_gui::QStandardItemModel;

use qt_core::CaseSensitivity;
use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;

use std::rc::Rc;
use std::sync::{Arc, RwLock};

use rpfm_error::{Result, ErrorKind};
use rpfm_lib::packfile::PackFileInfo;
use rpfm_lib::packfile::packedfile::PackedFileInfo;
use rpfm_lib::packedfile::PackedFileType;
use rpfm_macros::*;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::*;
use crate::locale::qtr;
use crate::pack_tree::PackTree;
use crate::packedfile_views::{BuildData, DataSource, PackedFileView, TreeViewOperation, View, ViewType};
use crate::packfile_contents_ui::PackFileContentsUI;

use self::slots::PackedFileAnimPackViewSlots;

mod connections;
pub mod slots;
mod shortcuts;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of an AnimPack PackedFile.
#[derive(GetRef)]
pub struct PackedFileAnimPackView {
    path: Arc<RwLock<Vec<String>>>,

    pack_tree_view: QBox<QTreeView>,
    pack_tree_model_filter: QBox<QSortFilterProxyModel>,

    pack_filter_line_edit: QBox<QLineEdit>,
    pack_filter_autoexpand_matches_button: QBox<QPushButton>,
    pack_filter_case_sensitive_button: QBox<QPushButton>,

    pack_expand_all: QBox<QAction>,
    pack_collapse_all: QBox<QAction>,

    anim_pack_tree_view: QBox<QTreeView>,
    anim_pack_tree_model_filter: QBox<QSortFilterProxyModel>,
    anim_pack_tree_model: QBox<QStandardItemModel>,

    anim_pack_filter_line_edit: QBox<QLineEdit>,
    anim_pack_filter_autoexpand_matches_button: QBox<QPushButton>,
    anim_pack_filter_case_sensitive_button: QBox<QPushButton>,

    anim_pack_expand_all: QBox<QAction>,
    anim_pack_collapse_all: QBox<QAction>,

    anim_pack_delete: QBox<QAction>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileAnimPackView`.
impl PackedFileAnimPackView {

    /// This function creates a new AnimPack View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_view: &mut PackedFileView,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
    ) -> Result<PackedFileInfo> {

        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFile(packed_file_view.get_path(), packed_file_view.get_data_source()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let ((anim_pack_file_info, anim_packed_file_info), packed_file_info) = match response {
            Response::AnimPackPackedFileInfo((data, packed_file_info)) => (data, packed_file_info),
            Response::Error(error) => return Err(error),
            Response::Unknown => return Err(ErrorKind::PackedFileTypeUnknown.into()),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let layout: QPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast();

        // Create and configure the left `TreeView`, AKA the open PackFile.
        let instructions = QLabel::from_q_string_q_widget(&qtr("animpack_view_instructions"), packed_file_view.get_mut_widget());
        let pack_tree_view = QTreeView::new_1a(packed_file_view.get_mut_widget());
        let tree_model = &pack_file_contents_ui.packfile_contents_tree_model;
        let pack_tree_model_filter = new_treeview_filter_safe(packed_file_view.get_mut_widget().static_upcast());
        pack_tree_model_filter.set_source_model(tree_model);
        pack_tree_view.set_model(&pack_tree_model_filter);
        pack_tree_view.set_header_hidden(true);
        pack_tree_view.set_animated(true);
        pack_tree_view.set_uniform_row_heights(true);
        pack_tree_view.set_selection_mode(SelectionMode::ExtendedSelection);
        pack_tree_view.set_expands_on_double_click(false);
        pack_tree_view.header().set_stretch_last_section(true);

        // Create and configure the widgets to control the `TreeView`s filter.
        let pack_filter_line_edit = QLineEdit::from_q_widget(packed_file_view.get_mut_widget());
        let pack_filter_autoexpand_matches_button = QPushButton::from_q_string_q_widget(&qtr("treeview_autoexpand"), packed_file_view.get_mut_widget());
        let pack_filter_case_sensitive_button = QPushButton::from_q_string_q_widget(&qtr("treeview_aai"), packed_file_view.get_mut_widget());
        pack_filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        pack_filter_line_edit.set_clear_button_enabled(true);
        pack_filter_autoexpand_matches_button.set_checkable(true);
        pack_filter_case_sensitive_button.set_checkable(true);

        // Create the extra actions for the TreeView.
        let pack_expand_all = QAction::from_q_string_q_object(&qtr("treeview_expand_all"), packed_file_view.get_mut_widget());
        let pack_collapse_all = QAction::from_q_string_q_object(&qtr("treeview_collapse_all"), packed_file_view.get_mut_widget());
        pack_tree_view.add_action(&pack_expand_all);
        pack_tree_view.add_action(&pack_collapse_all);

        // Add everything to the main widget's Layout.
        layout.add_widget_5a(&instructions, 0, 0, 1, 4);
        layout.add_widget_5a(&pack_tree_view, 1, 0, 1, 2);
        layout.add_widget_5a(&pack_filter_line_edit, 2, 0, 1, 2);
        layout.add_widget_5a(&pack_filter_autoexpand_matches_button, 3, 0, 1, 1);
        layout.add_widget_5a(&pack_filter_case_sensitive_button, 3, 1, 1, 1);

        // Create and configure the right `TreeView`, AKA the AnimPack.
        let anim_pack_tree_view = QTreeView::new_1a(packed_file_view.get_mut_widget());
        let anim_pack_tree_model = QStandardItemModel::new_1a(packed_file_view.get_mut_widget());
        let anim_pack_tree_model_filter = new_treeview_filter_safe(packed_file_view.get_mut_widget().static_upcast());
        anim_pack_tree_model_filter.set_source_model(&anim_pack_tree_model);
        anim_pack_tree_view.set_model(&anim_pack_tree_model_filter);
        anim_pack_tree_view.set_header_hidden(true);
        anim_pack_tree_view.set_animated(true);
        anim_pack_tree_view.set_uniform_row_heights(true);
        anim_pack_tree_view.set_selection_mode(SelectionMode::ExtendedSelection);
        anim_pack_tree_view.set_expands_on_double_click(false);
        anim_pack_tree_view.header().set_stretch_last_section(false);

        let mut build_data = BuildData::new();
        build_data.data = Some((anim_pack_file_info, anim_packed_file_info));
        build_data.editable = false;
        anim_pack_tree_view.update_treeview(true, TreeViewOperation::Build(build_data), DataSource::PackFile);

        // Create and configure the widgets to control the `TreeView`s filter.
        let anim_pack_filter_line_edit = QLineEdit::from_q_widget(packed_file_view.get_mut_widget());
        let anim_pack_filter_autoexpand_matches_button = QPushButton::from_q_string_q_widget(&qtr("treeview_autoexpand"), packed_file_view.get_mut_widget());
        let anim_pack_filter_case_sensitive_button = QPushButton::from_q_string_q_widget(&qtr("treeview_aai"), packed_file_view.get_mut_widget());
        anim_pack_filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        anim_pack_filter_line_edit.set_clear_button_enabled(true);
        anim_pack_filter_autoexpand_matches_button.set_checkable(true);
        anim_pack_filter_case_sensitive_button.set_checkable(true);

        // Create the extra actions for the TreeView.
        let anim_pack_expand_all = QAction::from_q_string_q_object(&qtr("treeview_expand_all"), packed_file_view.get_mut_widget());
        let anim_pack_collapse_all = QAction::from_q_string_q_object(&qtr("treeview_collapse_all"), packed_file_view.get_mut_widget());
        let anim_pack_delete = QAction::from_q_string_q_object(&qtr("treeview_animpack_delete"), packed_file_view.get_mut_widget());

        anim_pack_tree_view.add_action(&anim_pack_expand_all);
        anim_pack_tree_view.add_action(&anim_pack_collapse_all);
        anim_pack_tree_view.add_action(&anim_pack_delete);

        // Add everything to the main widget's Layout.
        layout.add_widget_5a(&anim_pack_tree_view, 1, 2, 1, 2);
        layout.add_widget_5a(&anim_pack_filter_line_edit, 2, 2, 1, 2);
        layout.add_widget_5a(&anim_pack_filter_autoexpand_matches_button, 3, 2, 1, 1);
        layout.add_widget_5a(&anim_pack_filter_case_sensitive_button, 3, 3, 1, 1);

        let packed_file_animpack_view = Arc::new(PackedFileAnimPackView {
            path: packed_file_view.get_path_raw(),

            pack_tree_view,
            pack_tree_model_filter,

            pack_filter_line_edit,
            pack_filter_autoexpand_matches_button,
            pack_filter_case_sensitive_button,

            pack_expand_all,
            pack_collapse_all,

            anim_pack_tree_view,
            anim_pack_tree_model_filter,
            anim_pack_tree_model,

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
        shortcuts::set_shortcuts(&packed_file_animpack_view);
        packed_file_view.view = ViewType::Internal(View::AnimPack(packed_file_animpack_view));
        packed_file_view.packed_file_type = PackedFileType::AnimPack;

        Ok(packed_file_info)
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: (PackFileInfo, Vec<PackedFileInfo>)) {
        let mut build_data = BuildData::new();
        build_data.data = Some(data);
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
