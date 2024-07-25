//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the view for Anim Fragment file.

use qt_widgets::QCheckBox;
use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QSpinBox;
use qt_widgets::QTableView;

use qt_core::QPtr;
use qt_core::QString;

use cpp_core::CppDeletable;

use anyhow::Result;
use getset::*;

use std::rc::Rc;
use std::sync::Arc;

use rpfm_lib::files::{anim_fragment_battle::AnimFragmentBattle, FileType};
use rpfm_lib::games::supported_games::*;
use rpfm_ui_common::locale::qtr;
use rpfm_ui_common::utils::*;

use crate::GAME_SELECTED;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packedfile_views::{AppUI, FileView, PackFileContentsUI, View, ViewType};
use crate::references_ui::ReferencesUI;
use crate::views::table::utils::get_table_from_view;
use crate::views::table::{TableType, TableView};

const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/anim_fragment_battle_view.ui";
const VIEW_RELEASE: &str = "ui/anim_fragment_battle_view.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Getters)]
#[getset(get = "pub")]
pub struct FileAnimFragmentBattleView {
    version_spinbox: QPtr<QSpinBox>,
    subversion_spinbox: QPtr<QSpinBox>,
    min_id_spinbox: QPtr<QSpinBox>,
    max_id_spinbox: QPtr<QSpinBox>,
    skeleton_name_line_edit: QPtr<QLineEdit>,
    table_name_line_edit: QPtr<QLineEdit>,
    mount_table_name_line_edit: QPtr<QLineEdit>,
    unmount_table_name_line_edit: QPtr<QLineEdit>,
    locomotion_graph_line_edit: QPtr<QLineEdit>,
    is_simple_flight_checkbox: QPtr<QCheckBox>,
    is_new_cavalry_tech_checkbox: QPtr<QCheckBox>,

    table: Arc<TableView>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl FileAnimFragmentBattleView {

    pub unsafe fn new_view(
        file_view: &mut FileView,
        app_ui: &Rc<AppUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
        data: AnimFragmentBattle
    ) -> Result<()> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(file_view.main_widget(), template_path)?;
        let layout: QPtr<QGridLayout> = file_view.main_widget().layout().static_downcast();
        layout.add_widget_5a(&main_widget, 0, 0, 1, 1);

        let version_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "version_label")?;
        let subversion_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "subversion_label")?;
        let min_id_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "min_id_label")?;
        let max_id_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "max_id_label")?;
        let skeleton_name_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "skeleton_name_label")?;
        let table_name_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "table_name_label")?;
        let mount_table_name_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "mount_table_name_label")?;
        let unmount_table_name_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "unmount_table_name_label")?;
        let locomotion_graph_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "locomotion_graph_label")?;
        let is_simple_flight_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "is_simple_flight_label")?;
        let is_new_cavalry_tech_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "is_new_cavalry_tech_label")?;

        version_label.set_text(&qtr("anim_fragment_version"));
        subversion_label.set_text(&qtr("anim_fragment_subversion"));
        min_id_label.set_text(&qtr("anim_fragment_min_id"));
        max_id_label.set_text(&qtr("anim_fragment_max_id"));
        skeleton_name_label.set_text(&qtr("anim_fragment_skeleton_name"));
        table_name_label.set_text(&qtr("anim_fragment_table_name"));
        mount_table_name_label.set_text(&qtr("anim_fragment_mount_table_name"));
        unmount_table_name_label.set_text(&qtr("anim_fragment_unmount_table_name"));
        locomotion_graph_label.set_text(&qtr("anim_fragment_locomotion_graph"));
        is_simple_flight_label.set_text(&qtr("anim_fragment_is_simple_flight"));
        is_new_cavalry_tech_label.set_text(&qtr("anim_fragment_is_new_cavalry_tech"));

        let version_spinbox: QPtr<QSpinBox> = find_widget(&main_widget.static_upcast(), "version_spinbox")?;
        let subversion_spinbox: QPtr<QSpinBox> = find_widget(&main_widget.static_upcast(), "subversion_spinbox")?;
        let min_id_spinbox: QPtr<QSpinBox> = find_widget(&main_widget.static_upcast(), "min_id_spinbox")?;
        let max_id_spinbox: QPtr<QSpinBox> = find_widget(&main_widget.static_upcast(), "max_id_spinbox")?;
        let skeleton_name_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "skeleton_name_line_edit")?;
        let table_name_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "table_name_line_edit")?;
        let mount_table_name_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "mount_table_name_line_edit")?;
        let unmount_table_name_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "unmount_table_name_line_edit")?;
        let locomotion_graph_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "locomotion_graph_line_edit")?;
        let is_simple_flight_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "is_simple_flight_checkbox")?;
        let is_new_cavalry_tech_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "is_new_cavalry_tech_checkbox")?;

        version_spinbox.set_maximum(i32::MAX);
        version_spinbox.set_minimum(i32::MIN);
        subversion_spinbox.set_maximum(i32::MAX);
        subversion_spinbox.set_minimum(i32::MIN);
        min_id_spinbox.set_maximum(i32::MAX);
        min_id_spinbox.set_minimum(i32::MIN);
        max_id_spinbox.set_maximum(i32::MAX);
        max_id_spinbox.set_minimum(i32::MIN);

        let entries_table_view: QPtr<QTableView> = find_widget(&main_widget.static_upcast(), "entries_table_view")?;

        let table_data = TableType::AnimFragmentBattle(data.to_table()?);
        let table = TableView::new_view(&main_widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, references_ui, table_data, Some(file_view.path_raw()), file_view.data_source.clone())?;

        let layout = main_widget.layout().static_downcast::<QGridLayout>();
        layout.replace_widget_2a(entries_table_view.as_ptr(), table.table_view().as_ptr());
        entries_table_view.delete();

        let view = Self {
            version_spinbox,
            subversion_spinbox,
            min_id_spinbox,
            max_id_spinbox,
            skeleton_name_line_edit,
            table_name_line_edit,
            mount_table_name_line_edit,
            unmount_table_name_line_edit,
            locomotion_graph_line_edit,
            is_simple_flight_checkbox,
            is_new_cavalry_tech_checkbox,

            table,
        };

        view.version_spinbox.set_value(*data.version() as i32);
        view.subversion_spinbox.set_value(*data.subversion() as i32);
        view.min_id_spinbox.set_value(*data.min_id() as i32);
        view.max_id_spinbox.set_value(*data.max_id() as i32);
        view.skeleton_name_line_edit.set_text(&QString::from_std_str(data.skeleton_name()));
        view.table_name_line_edit.set_text(&QString::from_std_str(data.table_name()));
        view.mount_table_name_line_edit.set_text(&QString::from_std_str(data.mount_table_name()));
        view.unmount_table_name_line_edit.set_text(&QString::from_std_str(data.unmount_table_name()));
        view.locomotion_graph_line_edit.set_text(&QString::from_std_str(data.locomotion_graph()));
        view.is_simple_flight_checkbox.set_checked(*data.is_simple_flight());
        view.is_new_cavalry_tech_checkbox.set_checked(*data.is_new_cavalry_tech());

        view.version_spinbox.set_enabled(false);
        view.subversion_spinbox.set_enabled(false);

        // Hide the items not relevant for the current game.
        let game = GAME_SELECTED.read().unwrap();
        if game.key() == KEY_WARHAMMER_3 {
            min_id_label.hide();
            view.min_id_spinbox.hide();

            max_id_label.hide();
            view.max_id_spinbox.hide();

            view.table.table_view().hide_column(10);
            view.table.table_view().hide_column(11);
            view.table.table_view().hide_column(12);
            view.table.table_view().hide_column(13);
            view.table.table_view().hide_column(14);
            view.table.table_view().hide_column(15);
            view.table.table_view().hide_column(16);

        } else if game.key() == KEY_WARHAMMER_2 || game.key() == KEY_TROY || game.key() == KEY_PHARAOH || game.key() == KEY_PHARAOH_DYNASTIES {
            subversion_label.hide();
            table_name_label.hide();
            unmount_table_name_label.hide();
            locomotion_graph_label.hide();
            is_simple_flight_label.hide();
            is_new_cavalry_tech_label.hide();

            view.subversion_spinbox.hide();
            view.table_name_line_edit.hide();
            view.unmount_table_name_line_edit.hide();
            view.locomotion_graph_line_edit.hide();
            view.is_simple_flight_checkbox.hide();
            view.is_new_cavalry_tech_checkbox.hide();

            view.table.table_view().hide_column(9);

        } else if game.key() == KEY_THREE_KINGDOMS {
            subversion_label.hide();
            view.subversion_spinbox.hide();

            min_id_label.hide();
            view.min_id_spinbox.hide();

            max_id_label.hide();
            view.max_id_spinbox.hide();

            locomotion_graph_label.hide();
            view.locomotion_graph_line_edit.hide();

            view.table.table_view().hide_column(10);
            view.table.table_view().hide_column(11);
            view.table.table_view().hide_column(12);
            view.table.table_view().hide_column(13);
            view.table.table_view().hide_column(14);
            view.table.table_view().hide_column(15);
            view.table.table_view().hide_column(16);
        }

        file_view.view_type = ViewType::Internal(View::AnimFragmentBattle(Arc::new(view)));
        file_view.file_type = FileType::AnimFragmentBattle;

        Ok(())
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: AnimFragmentBattle) -> Result<()> {
        self.version_spinbox.set_value(*data.version() as i32);
        self.subversion_spinbox.set_value(*data.subversion() as i32);
        self.min_id_spinbox.set_value(*data.min_id() as i32);
        self.max_id_spinbox.set_value(*data.max_id() as i32);
        self.skeleton_name_line_edit.set_text(&QString::from_std_str(data.skeleton_name()));
        self.table_name_line_edit.set_text(&QString::from_std_str(data.table_name()));
        self.mount_table_name_line_edit.set_text(&QString::from_std_str(data.mount_table_name()));
        self.unmount_table_name_line_edit.set_text(&QString::from_std_str(data.unmount_table_name()));
        self.locomotion_graph_line_edit.set_text(&QString::from_std_str(data.locomotion_graph()));
        self.is_simple_flight_checkbox.set_checked(*data.is_simple_flight());
        self.is_new_cavalry_tech_checkbox.set_checked(*data.is_new_cavalry_tech());

        self.table.reload_view(TableType::AnimFragmentBattle(data.to_table()?));

        Ok(())
    }

    pub unsafe fn save_view(&self) -> Result<AnimFragmentBattle> {
        let mut data = AnimFragmentBattle::default();
        data.set_version(self.version_spinbox.value() as u32);
        data.set_subversion(self.subversion_spinbox.value() as u32);
        data.set_min_id(self.min_id_spinbox.value() as u32);
        data.set_max_id(self.max_id_spinbox.value() as u32);
        data.set_skeleton_name(self.skeleton_name_line_edit.text().to_std_string());
        data.set_table_name(self.table_name_line_edit.text().to_std_string());
        data.set_mount_table_name(self.mount_table_name_line_edit.text().to_std_string());
        data.set_unmount_table_name(self.unmount_table_name_line_edit.text().to_std_string());
        data.set_locomotion_graph(self.locomotion_graph_line_edit.text().to_std_string());
        data.set_is_simple_flight(self.is_simple_flight_checkbox.is_checked());
        data.set_is_new_cavalry_tech(self.is_new_cavalry_tech_checkbox.is_checked());

        let table = get_table_from_view(&self.table.table_model().static_upcast(), &self.table.table_definition())?;
        data.set_entries(AnimFragmentBattle::from_table(&table)?);

        Ok(data)
    }
}
