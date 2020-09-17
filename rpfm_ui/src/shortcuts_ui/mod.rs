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

use qt_core::QBox;
use qt_core::Orientation;
use qt_core::QString;
use qt_core::QVariant;
use qt_core::QPtr;

use cpp_core::CastInto;
use cpp_core::Ptr;

use std::rc::Rc;

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
pub struct ShortcutsUI {
    dialog: QBox<QDialog>,

    shortcuts_table: QBox<QTreeView>,
    shortcuts_model: QBox<QStandardItemModel>,

    restore_default_button: QPtr<QPushButton>,
    cancel_button: QPtr<QPushButton>,
    accept_button: QPtr<QPushButton>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `ShortcutsUI`.
impl ShortcutsUI {

    /// This function creates a ***ShortcutsUI*** dialog, execute it, and returns a new `Shortcuts`, or `None` if you close/cancel the dialog.
    pub unsafe fn new(parent: impl CastInto<Ptr<QWidget>>) -> Option<Shortcuts> {
        let ui = Rc::new(Self::new_with_parent(parent));
        let slots = ShortcutsUISlots::new(&ui);
        connections::set_connections(&ui, &slots);
        Self::load(&ui, &UI_STATE.get_shortcuts());

        if ui.dialog.exec() == 1 { Some(Self::save(&ui)) }
        else { None }
    }

    /// This function creates the entire `ShortcutsUI` Window and shows it.
    pub unsafe fn new_with_parent(parent: impl CastInto<Ptr<QWidget>>) -> Self {

        // Create the Shortcuts Dialog and configure it.
        let dialog = QDialog::new_1a(parent);
        dialog.set_window_title(&qtr("shortcut_title"));
        dialog.set_modal(true);
        dialog.resize_2a(1100, 700);

        // Create the main Grid and add the shortcuts TreeView.
        let main_grid = create_grid_layout(dialog.static_upcast());
        let shortcuts_table = QTreeView::new_0a();
        let shortcuts_filter = new_treeview_filter_safe(shortcuts_table.static_upcast());
        let shortcuts_model = QStandardItemModel::new_0a();

        shortcuts_filter.set_source_model(&shortcuts_model);
        shortcuts_table.set_model(&shortcuts_filter);

        shortcuts_table.set_sorting_enabled(false);
        shortcuts_table.header().set_stretch_last_section(true);
        main_grid.add_widget_5a(& shortcuts_table, 0, 0, 1, 1);

        // Create the bottom buttons and add them to the Dialog.
        let button_box = QDialogButtonBox::new();
        let restore_default_button = button_box.add_button_standard_button(q_dialog_button_box::StandardButton::RestoreDefaults);
        let cancel_button = button_box.add_button_standard_button(q_dialog_button_box::StandardButton::Cancel);
        let accept_button = button_box.add_button_standard_button(q_dialog_button_box::StandardButton::Save);
        main_grid.add_widget_5a(button_box.into_ptr(), 1, 0, 1, 1);

        Self {
            dialog,
            shortcuts_table,
            shortcuts_model,
            restore_default_button,
            cancel_button,
            accept_button,
        }
    }

    /// This function loads the data from the `Shortcuts` struct to the `ShortcutsUI`.
    pub unsafe fn load(shortcuts_ui: &Rc<Self>, shortcuts: &Shortcuts) {

        // Clear all the models, just in case this is a restore default operation.
        shortcuts_ui.shortcuts_model.clear();

        // Just add in mass the shortcuts to the Model, separated in sections.
        {
            let menu_bar_packfile_parent = QListOfQStandardItem::new();
            let section = QStandardItem::new();
            let fill1 = QStandardItem::new();
            section.set_text(&qtr("menu_bar_packfile_section"));
            section.set_editable(false);
            fill1.set_editable(false);
            for (key, value) in shortcuts.menu_bar_packfile.iter() {
                let row_list = QListOfQStandardItem::new();
                let key = QStandardItem::from_q_string(&QString::from_std_str(key));
                let value = QStandardItem::from_q_string(&QString::from_std_str(value));
                key.set_editable(false);

                row_list.append_q_standard_item(&mut key.into_ptr().as_mut_raw_ptr());
                row_list.append_q_standard_item(&mut value.into_ptr().as_mut_raw_ptr());

                section.append_row_q_list_of_q_standard_item(row_list.as_ref());
            }

            menu_bar_packfile_parent.append_q_standard_item(&mut section.into_ptr().as_mut_raw_ptr());
            menu_bar_packfile_parent.append_q_standard_item(&mut fill1.into_ptr().as_mut_raw_ptr());

            shortcuts_ui.shortcuts_model.append_row_q_list_of_q_standard_item(menu_bar_packfile_parent.as_ref());
        }

        {
            let menu_bar_packfile_parent = QListOfQStandardItem::new().into_ptr();
            let section = QStandardItem::new();
            let fill1 = QStandardItem::new();
            section.set_text(&qtr("menu_bar_mymod_section"));
            section.set_editable(false);
            fill1.set_editable(false);
            for (key, value) in shortcuts.menu_bar_mymod.iter() {
                let row_list = QListOfQStandardItem::new();
                let key = QStandardItem::from_q_string(&QString::from_std_str(key));
                let value = QStandardItem::from_q_string(&QString::from_std_str(value));
                key.set_editable(false);

                row_list.append_q_standard_item(&mut key.into_ptr().as_mut_raw_ptr());
                row_list.append_q_standard_item(&mut value.into_ptr().as_mut_raw_ptr());

                section.append_row_q_list_of_q_standard_item(row_list.as_ref());
            }

            menu_bar_packfile_parent.append_q_standard_item(&mut section.into_ptr().as_mut_raw_ptr());
            menu_bar_packfile_parent.append_q_standard_item(&mut fill1.into_ptr().as_mut_raw_ptr());

            shortcuts_ui.shortcuts_model.append_row_q_list_of_q_standard_item(menu_bar_packfile_parent.as_ref().unwrap());
        }

        {
            let menu_bar_packfile_parent = QListOfQStandardItem::new().into_ptr();
            let section = QStandardItem::new();
            let fill1 = QStandardItem::new();
            section.set_text(&qtr("menu_bar_view_section"));
            section.set_editable(false);
            fill1.set_editable(false);
            for (key, value) in shortcuts.menu_bar_view.iter() {
                let row_list = QListOfQStandardItem::new();
                let key = QStandardItem::from_q_string(&QString::from_std_str(key));
                let value = QStandardItem::from_q_string(&QString::from_std_str(value));
                key.set_editable(false);

                row_list.append_q_standard_item(&mut key.into_ptr().as_mut_raw_ptr());
                row_list.append_q_standard_item(&mut value.into_ptr().as_mut_raw_ptr());

                section.append_row_q_list_of_q_standard_item(row_list.as_ref());
            }

            menu_bar_packfile_parent.append_q_standard_item(&mut section.into_ptr().as_mut_raw_ptr());
            menu_bar_packfile_parent.append_q_standard_item(&mut fill1.into_ptr().as_mut_raw_ptr());

            shortcuts_ui.shortcuts_model.append_row_q_list_of_q_standard_item(menu_bar_packfile_parent.as_ref().unwrap());
        }

        {
            let menu_bar_packfile_parent = QListOfQStandardItem::new().into_ptr();
            let section = QStandardItem::new();
            let fill1 = QStandardItem::new();
            section.set_text(&qtr("menu_bar_game_selected_section"));
            section.set_editable(false);
            fill1.set_editable(false);
            for (key, value) in shortcuts.menu_bar_game_selected.iter() {
                let row_list = QListOfQStandardItem::new();
                let key = QStandardItem::from_q_string(&QString::from_std_str(key));
                let value = QStandardItem::from_q_string(&QString::from_std_str(value));
                key.set_editable(false);

                row_list.append_q_standard_item(&mut key.into_ptr().as_mut_raw_ptr());
                row_list.append_q_standard_item(&mut value.into_ptr().as_mut_raw_ptr());

                section.append_row_q_list_of_q_standard_item(row_list.as_ref());
            }

            menu_bar_packfile_parent.append_q_standard_item(&mut section.into_ptr().as_mut_raw_ptr());
            menu_bar_packfile_parent.append_q_standard_item(&mut fill1.into_ptr().as_mut_raw_ptr());

            shortcuts_ui.shortcuts_model.append_row_q_list_of_q_standard_item(menu_bar_packfile_parent.as_ref().unwrap());
        }

        {
            let menu_bar_packfile_parent = QListOfQStandardItem::new().into_ptr();
            let section = QStandardItem::new();
            let fill1 = QStandardItem::new();
            section.set_text(&qtr("menu_bar_about_section"));
            section.set_editable(false);
            fill1.set_editable(false);
            for (key, value) in shortcuts.menu_bar_about.iter() {
                let row_list = QListOfQStandardItem::new();
                let key = QStandardItem::from_q_string(&QString::from_std_str(key));
                let value = QStandardItem::from_q_string(&QString::from_std_str(value));
                key.set_editable(false);

                row_list.append_q_standard_item(&mut key.into_ptr().as_mut_raw_ptr());
                row_list.append_q_standard_item(&mut value.into_ptr().as_mut_raw_ptr());

                section.append_row_q_list_of_q_standard_item(row_list.as_ref());
            }

            menu_bar_packfile_parent.append_q_standard_item(&mut section.into_ptr().as_mut_raw_ptr());
            menu_bar_packfile_parent.append_q_standard_item(&mut fill1.into_ptr().as_mut_raw_ptr());

            shortcuts_ui.shortcuts_model.append_row_q_list_of_q_standard_item(menu_bar_packfile_parent.as_ref().unwrap());
        }

        {
            let menu_bar_packfile_parent = QListOfQStandardItem::new().into_ptr();
            let section = QStandardItem::new();
            let fill1 = QStandardItem::new();
            section.set_text(&qtr("packfile_contents_tree_view_section"));
            section.set_editable(false);
            fill1.set_editable(false);
            for (key, value) in shortcuts.packfile_contents_tree_view.iter() {
                let row_list = QListOfQStandardItem::new();
                let key = QStandardItem::from_q_string(&QString::from_std_str(key));
                let value = QStandardItem::from_q_string(&QString::from_std_str(value));
                key.set_editable(false);

                row_list.append_q_standard_item(&mut key.into_ptr().as_mut_raw_ptr());
                row_list.append_q_standard_item(&mut value.into_ptr().as_mut_raw_ptr());

                section.append_row_q_list_of_q_standard_item(row_list.as_ref());
            }

            menu_bar_packfile_parent.append_q_standard_item(&mut section.into_ptr().as_mut_raw_ptr());
            menu_bar_packfile_parent.append_q_standard_item(&mut fill1.into_ptr().as_mut_raw_ptr());

            shortcuts_ui.shortcuts_model.append_row_q_list_of_q_standard_item(menu_bar_packfile_parent.as_ref().unwrap());
        }

        {
            let menu_bar_packfile_parent = QListOfQStandardItem::new().into_ptr();
            let section = QStandardItem::new();
            let fill1 = QStandardItem::new();
            section.set_text(&qtr("packed_file_table_section"));
            section.set_editable(false);
            fill1.set_editable(false);
            for (key, value) in shortcuts.packed_file_table.iter() {
                let row_list = QListOfQStandardItem::new();
                let key = QStandardItem::from_q_string(&QString::from_std_str(key));
                let value = QStandardItem::from_q_string(&QString::from_std_str(value));
                key.set_editable(false);

                row_list.append_q_standard_item(&mut key.into_ptr().as_mut_raw_ptr());
                row_list.append_q_standard_item(&mut value.into_ptr().as_mut_raw_ptr());

                section.append_row_q_list_of_q_standard_item(row_list.as_ref());
            }

            menu_bar_packfile_parent.append_q_standard_item(&mut section.into_ptr().as_mut_raw_ptr());
            menu_bar_packfile_parent.append_q_standard_item(&mut fill1.into_ptr().as_mut_raw_ptr());

            shortcuts_ui.shortcuts_model.append_row_q_list_of_q_standard_item(menu_bar_packfile_parent.as_ref().unwrap());
        }

        {
            let menu_bar_packfile_parent = QListOfQStandardItem::new().into_ptr();
            let section = QStandardItem::new();
            let fill1 = QStandardItem::new();
            section.set_text(&qtr("packed_file_decoder_section"));
            section.set_editable(false);
            fill1.set_editable(false);
            for (key, value) in shortcuts.packed_file_decoder.iter() {
                let row_list = QListOfQStandardItem::new();
                let key = QStandardItem::from_q_string(&QString::from_std_str(key));
                let value = QStandardItem::from_q_string(&QString::from_std_str(value));
                key.set_editable(false);

                row_list.append_q_standard_item(&mut key.into_ptr().as_mut_raw_ptr());
                row_list.append_q_standard_item(&mut value.into_ptr().as_mut_raw_ptr());

                section.append_row_q_list_of_q_standard_item(row_list.as_ref());
            }

            menu_bar_packfile_parent.append_q_standard_item(&mut section.into_ptr().as_mut_raw_ptr());
            menu_bar_packfile_parent.append_q_standard_item(&mut fill1.into_ptr().as_mut_raw_ptr());

            shortcuts_ui.shortcuts_model.append_row_q_list_of_q_standard_item(menu_bar_packfile_parent.as_ref().unwrap());
        }

        // Rename the columns and expand all.
        shortcuts_ui.shortcuts_model.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&qtr("shortcut_section_action")));
        shortcuts_ui.shortcuts_model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&qtr("shortcut_text")));
        shortcuts_ui.shortcuts_table.expand_all();
        shortcuts_ui.shortcuts_table.header().resize_sections(ResizeMode::ResizeToContents);
    }

    /// This function gets the data from the `ShortcutsUI` and returns a `Shortcuts` struct with that data in it.
    pub unsafe fn save(shortcuts_ui: &Rc<Self>) -> Shortcuts {

        // Create a new Shortcuts struct to populate it wit the contents of the model.
        let mut shortcuts = Shortcuts::new();
        let shortcuts_model = shortcuts_ui.shortcuts_model.as_ref().unwrap();
        let root = shortcuts_model.invisible_root_item().as_ref().unwrap();

        let menu_bar_packfile_section_title = tr("menu_bar_packfile_section");
        let menu_bar_mymod_section_title = tr("menu_bar_mymod_section");
        let menu_bar_view_section_title = tr("menu_bar_view_section");
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
                else if section_text == menu_bar_view_section_title { &mut shortcuts.menu_bar_view }
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
