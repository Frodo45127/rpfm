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
This module contains the code to build/use the ***Shortcuts*** UI.
!*/

use qt_widgets::QDialog;
use qt_widgets::q_dialog_button_box;
use qt_widgets::QDialogButtonBox;
use qt_widgets::q_header_view::ResizeMode;
use qt_widgets::QPushButton;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::QAbstractItemModel;
use qt_core::QObject;
use qt_core::QSortFilterProxyModel;
use qt_core::Orientation;
use qt_core::QString;
use qt_core::QVariant;

use cpp_core::CastInto;
use cpp_core::MutPtr;

use crate::ffi::new_treeview_filter_safe;
use crate::locale::{qtr, tr};
use crate::ui_state::shortcuts::Shortcuts;
use crate::utils::create_grid_layout;
use crate::UI_STATE;
use self::slots::ShortcutsUISlots;

mod connections;
mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct holds all the widgets used in the Shortcuts Window.
#[derive(Clone)]
pub struct ShortcutsUI {
    dialog: MutPtr<QDialog>,

    shortcuts_table: MutPtr<QTreeView>,
    shortcuts_model: MutPtr<QStandardItemModel>,
    shortcuts_filter: MutPtr<QSortFilterProxyModel>,

    restore_default_button: MutPtr<QPushButton>,
    cancel_button: MutPtr<QPushButton>,
    accept_button: MutPtr<QPushButton>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `ShortcutsUI`.
impl ShortcutsUI {

    /// This function creates a ***ShortcutsUI*** dialog, execute it, and returns a new `Shortcuts`, or `None` if you close/cancel the dialog.
    pub unsafe fn new(parent: impl CastInto<MutPtr<QWidget>>) -> Option<Shortcuts> {
        let mut ui = Self::new_with_parent(parent);
        let slots = ShortcutsUISlots::new(&ui);
        connections::set_connections(&ui, &slots);
        ui.load(&UI_STATE.get_shortcuts());

        if ui.dialog.exec() == 1 { Some(ui.save()) }
        else { None }
    }

    /// This function creates the entire `ShortcutsUI` Window and shows it.
    pub unsafe fn new_with_parent(parent: impl CastInto<MutPtr<QWidget>>) -> Self {

        // Create the Shortcuts Dialog and configure it.
        let mut dialog = QDialog::new_1a(parent).into_ptr();
        dialog.set_window_title(&qtr("shortcut_title"));
        dialog.set_modal(true);
        dialog.resize_2a(1100, 700);

        // Create the main Grid and add the shortcuts TreeView.
        let mut main_grid = create_grid_layout(dialog.static_upcast_mut());
        let mut shortcuts_table = QTreeView::new_0a();
        let mut shortcuts_filter = new_treeview_filter_safe(&mut shortcuts_table);
        let mut shortcuts_model = QStandardItemModel::new_0a();

        shortcuts_filter.set_source_model(&mut shortcuts_model);
        shortcuts_table.set_model(shortcuts_filter);

        shortcuts_table.set_sorting_enabled(false);
        shortcuts_table.header().set_stretch_last_section(true);
        main_grid.add_widget_5a(&mut shortcuts_table, 0, 0, 1, 1);

        // Create the bottom buttons and add them to the Dialog.
        let mut button_box = QDialogButtonBox::new();
        let restore_default_button = button_box.add_button_standard_button(q_dialog_button_box::StandardButton::RestoreDefaults);
        let cancel_button = button_box.add_button_standard_button(q_dialog_button_box::StandardButton::Cancel);
        let accept_button = button_box.add_button_standard_button(q_dialog_button_box::StandardButton::Save);
        main_grid.add_widget_5a(&mut button_box, 1, 0, 1, 1);

        Self {
            dialog,
            shortcuts_table: shortcuts_table.into_ptr(),
            shortcuts_model: shortcuts_model.into_ptr(),
            shortcuts_filter,
            restore_default_button,
            cancel_button,
            accept_button,
        }
    }

    /// This function loads the data from the `Shortcuts` struct to the `ShortcutsUI`.
    pub unsafe fn load(&mut self, shortcuts: &Shortcuts) {

        // Clear all the models, just in case this is a restore default operation.
        self.shortcuts_model.clear();
/*
        // Just add in mass the shortcuts to the Model, separated in sections.
        {
            let mut menu_bar_packfile_parent = QListOfQStandardItem::new();
            let mut section = QStandardItem::new();
            let mut fill1 = QStandardItem::new();
            section.set_text(&qtr("menu_bar_packfile_section"));
            section.set_editable(false);
            fill1.set_editable(false);
            for (key, value) in shortcuts.menu_bar_packfile.iter() {
                let mut row_list = QListOfQStandardItem::new();
                unsafe { row_list.append_unsafe(&QStandardItem::from_q_string(&QString::from_std_str(key))); }
                unsafe { row_list.append_unsafe(&QStandardItem::from_q_string(&QString::from_std_str(value))); }
                unsafe { row_list.first().set_editable(false); }
                section.append_row_q_list_of_q_standard_item(&row_list);
            }

            unsafe { menu_bar_packfile_parent.append_unsafe(&section); }
            unsafe { menu_bar_packfile_parent.append_unsafe(&fill1); }
            self.shortcuts_model.append_row_q_list_of_q_standard_item(&menu_bar_packfile_parent);
        }

        {
            let mut menu_bar_packfile_parent = QListOfQStandardItem::new();
            let mut section = QStandardItem::new();
            let mut fill1 = QStandardItem::new();
            section.set_text(&qtr("menu_bar_mymod_section"));
            section.set_editable(false);
            fill1.set_editable(false);
            for (key, value) in shortcuts.menu_bar_mymod.iter() {
                let mut row_list = QListOfQStandardItem::new();
                unsafe { row_list.append_unsafe(&QStandardItem::from_q_string(&QString::from_std_str(key))); }
                unsafe { row_list.append_unsafe(&QStandardItem::from_q_string(&QString::from_std_str(value))); }
                unsafe { row_list.first().set_editable(false); }
                section.append_row_q_list_of_q_standard_item(&row_list);
            }

            unsafe { menu_bar_packfile_parent.append_unsafe(&section); }
            unsafe { menu_bar_packfile_parent.append_unsafe(&fill1); }
            self.shortcuts_model.append_row_q_list_of_q_standard_item(&menu_bar_packfile_parent);
        }

        {
            let mut menu_bar_packfile_parent = QListOfQStandardItem::new();
            let mut section = QStandardItem::new();
            let mut fill1 = QStandardItem::new();
            section.set_text(&qtr("menu_bar_game_selected_section"));
            section.set_editable(false);
            fill1.set_editable(false);
            for (key, value) in shortcuts.menu_bar_game_selected.iter() {
                let mut row_list = QListOfQStandardItem::new();
                unsafe { row_list.append_unsafe(&QStandardItem::from_q_string(&QString::from_std_str(key))); }
                unsafe { row_list.append_unsafe(&QStandardItem::from_q_string(&QString::from_std_str(value))); }
                unsafe { row_list.first().set_editable(false); }
                section.append_row_q_list_of_q_standard_item(&row_list);
            }

            unsafe { menu_bar_packfile_parent.append_unsafe(&section); }
            unsafe { menu_bar_packfile_parent.append_unsafe(&fill1); }
            self.shortcuts_model.append_row_q_list_of_q_standard_item(&menu_bar_packfile_parent);
        }

        {
            let mut menu_bar_packfile_parent = QListOfQStandardItem::new();
            let mut section = QStandardItem::new();
            let mut fill1 = QStandardItem::new();
            section.set_text(&qtr("menu_bar_about_section"));
            section.set_editable(false);
            fill1.set_editable(false);
            for (key, value) in shortcuts.menu_bar_about.iter() {
                let mut row_list = QListOfQStandardItem::new();
                unsafe { row_list.append_unsafe(&QStandardItem::from_q_string(&QString::from_std_str(key))); }
                unsafe { row_list.append_unsafe(&QStandardItem::from_q_string(&QString::from_std_str(value))); }
                unsafe { row_list.first().set_editable(false); }
                section.append_row_q_list_of_q_standard_item(&row_list);
            }

            unsafe { menu_bar_packfile_parent.append_unsafe(&section); }
            unsafe { menu_bar_packfile_parent.append_unsafe(&fill1); }
            self.shortcuts_model.append_row_q_list_of_q_standard_item(&menu_bar_packfile_parent);
        }

        {
            let mut menu_bar_packfile_parent = QListOfQStandardItem::new();
            let mut section = QStandardItem::new();
            let mut fill1 = QStandardItem::new();
            section.set_text(&qtr("packfile_contents_tree_view_section"));
            section.set_editable(false);
            fill1.set_editable(false);
            for (key, value) in shortcuts.packfile_contents_tree_view.iter() {
                let mut row_list = QListOfQStandardItem::new();
                unsafe { row_list.append_unsafe(&QStandardItem::from_q_string(&QString::from_std_str(key))); }
                unsafe { row_list.append_unsafe(&QStandardItem::from_q_string(&QString::from_std_str(value))); }
                unsafe { row_list.first().set_editable(false); }
                section.append_row_q_list_of_q_standard_item(&row_list);
            }

            unsafe { menu_bar_packfile_parent.append_unsafe(&section); }
            unsafe { menu_bar_packfile_parent.append_unsafe(&fill1); }
            self.shortcuts_model.append_row_q_list_of_q_standard_item(&menu_bar_packfile_parent);
        }

        {
            let mut menu_bar_packfile_parent = QListOfQStandardItem::new();
            let mut section = QStandardItem::new();
            let mut fill1 = QStandardItem::new();
            section.set_text(&qtr("packed_file_table_section"));
            section.set_editable(false);
            fill1.set_editable(false);
            for (key, value) in shortcuts.packed_file_table.iter() {
                let mut row_list = QListOfQStandardItem::new();
                unsafe { row_list.append_unsafe(&QStandardItem::from_q_string(&QString::from_std_str(key))); }
                unsafe { row_list.append_unsafe(&QStandardItem::from_q_string(&QString::from_std_str(value))); }
                unsafe { row_list.first().set_editable(false); }
                section.append_row_q_list_of_q_standard_item(&row_list);
            }

            unsafe { menu_bar_packfile_parent.append_unsafe(&section); }
            unsafe { menu_bar_packfile_parent.append_unsafe(&fill1); }
            self.shortcuts_model.append_row_q_list_of_q_standard_item(&menu_bar_packfile_parent);
        }

        {
            let mut menu_bar_packfile_parent = QListOfQStandardItem::new();
            let mut section = QStandardItem::new();
            let mut fill1 = QStandardItem::new();
            section.set_text(&qtr("packed_file_decoder_section"));
            section.set_editable(false);
            fill1.set_editable(false);
            for (key, value) in shortcuts.packed_file_decoder.iter() {
                let mut row_list = QListOfQStandardItem::new();
                unsafe { row_list.append_unsafe(&QStandardItem::from_q_string(&QString::from_std_str(key))); }
                unsafe { row_list.append_unsafe(&QStandardItem::from_q_string(&QString::from_std_str(value))); }
                unsafe { row_list.first().set_editable(false); }
                section.append_row_q_list_of_q_standard_item(&row_list);
            }

            unsafe { menu_bar_packfile_parent.append_unsafe(&section); }
            unsafe { menu_bar_packfile_parent.append_unsafe(&fill1); }
            self.shortcuts_model.append_row_q_list_of_q_standard_item(&menu_bar_packfile_parent);
        }
*/
        // Rename the columns and expand all.
        self.shortcuts_model.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&qtr("shortcut_section_action")));
        self.shortcuts_model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&qtr("shortcut_text")));
        self.shortcuts_table.expand_all();
        self.shortcuts_table.header().resize_sections(ResizeMode::ResizeToContents);
    }

    /// This function gets the data from the `ShortcutsUI` and returns a `Shortcuts` struct with that data in it.
    pub unsafe fn save(&self) -> Shortcuts {

        // Create a new Shortcuts struct to populate it wit the contents of the model.
        let mut shortcuts = Shortcuts::new();
        let shortcuts_model = self.shortcuts_model.as_ref().unwrap();
        let root = shortcuts_model.invisible_root_item().as_ref().unwrap();

        let menu_bar_packfile_section_title = tr("menu_bar_packfile_section");
        let menu_bar_mymod_section_title = tr("menu_bar_mymod_section");
        let menu_bar_game_selected_section_title = tr("menu_bar_game_selected_section");
        let menu_bar_about_section_title = tr("menu_bar_about_section");
        let packfile_contents_tree_view_section_title = tr("packfile_contents_tree_view_section");
        let packed_file_table_section_title = tr("packed_file_table_section");
        let packed_file_decoder_section_title = tr("packed_file_decoder_section");

        for index in 0..root.row_count() {
            let section = root.child_1a(index).as_ref().unwrap();
            let section_text = section.text().to_std_string();
            let map = if section_text == menu_bar_packfile_section_title { &mut shortcuts.menu_bar_packfile }
                else if section_text == menu_bar_mymod_section_title { &mut shortcuts.menu_bar_mymod }
                else if section_text == menu_bar_game_selected_section_title { &mut shortcuts.menu_bar_game_selected }
                else if section_text == menu_bar_about_section_title { &mut shortcuts.menu_bar_about }
                else if section_text == packfile_contents_tree_view_section_title { &mut shortcuts.packfile_contents_tree_view }
                else if section_text == packed_file_table_section_title { &mut shortcuts.packed_file_table }
                else if section_text == packed_file_decoder_section_title { &mut shortcuts.packed_file_decoder }
                else { panic!("WTF?!! YOU ARE NOT SUPPOSED TO MANUALLY DO WEIRD STUFF WITH THE RON FILE!!!") };

            for index in 0..section.row_count() {
                let key = section.child_2a(index, 0).as_ref().unwrap().text().to_std_string();
                let value = section.child_2a(index, 1).as_ref().unwrap().text().to_std_string();
                map.insert(key, value);
            }
        }

        shortcuts
    }
}
