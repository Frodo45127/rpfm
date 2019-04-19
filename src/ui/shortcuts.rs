//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// Here is all the stuff related to the Shortcuts window. Keep in mind this window is just visual.

use qt_widgets::dialog::Dialog;
use qt_widgets::dialog_button_box;
use qt_widgets::dialog_button_box::DialogButtonBox;
use qt_widgets::group_box::GroupBox;
use qt_widgets::table_view::TableView;
use qt_widgets::widget::Widget;

use qt_gui::list::ListStandardItemMutPtr;
use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::variant::Variant;
use qt_core::qt::Orientation;

use cpp_utils::StaticCast;

use super::*;
use crate::QString;
use crate::settings::shortcuts::Shortcuts;

/// ShortcutsDialog struct. To hold the TreeViews for easy loading/saving.
pub struct ShortcutsDialog {
    menu_bar_packfile: *mut StandardItemModel,
    menu_bar_about: *mut StandardItemModel,
    tree_view: *mut StandardItemModel,
    pack_files_list: *mut StandardItemModel,
    packed_files_table: *mut StandardItemModel,
    db_decoder_fields: *mut StandardItemModel,
    db_decoder_definitions: *mut StandardItemModel,
}

// Implementation of `ShortcutsDialog`.
impl ShortcutsDialog {

    /// This function creates the entire `Shortcuts` Window and shows it.
    pub fn create_shortcuts_dialog(
        window: *mut Dialog,
    ) -> Option<Shortcuts> {

        //-------------------------------------------------------------------------------------------//
        // Creating the Shortcuts Dialog...
        //-------------------------------------------------------------------------------------------//

        // Create the Shortcuts Dialog and configure it.
        let dialog = unsafe { Dialog::new_unsafe(window as *mut Widget).into_raw() };
        unsafe { dialog.as_mut().unwrap().set_window_title(&QString::from_std_str("Shortcuts")); }
        unsafe { dialog.as_mut().unwrap().set_modal(true); }
        unsafe { dialog.as_mut().unwrap().resize((1100, 700)); }

        // Create the main Grid.
        let main_grid = create_grid_layout_unsafe(dialog as *mut Widget);

        // Create the `MenuBar` Frame.
        let menu_bar_frame = GroupBox::new(&QString::from_std_str("Menu Bar")).into_raw();
        let menu_bar_grid = create_grid_layout_unsafe(menu_bar_frame as *mut Widget);

        // Create the TreeView Context Menu Frame.
        let tree_view_context_menu_frame = GroupBox::new(&QString::from_std_str("TreeView's Context Menu")).into_raw();
        let tree_view_context_menu_grid = create_grid_layout_unsafe(tree_view_context_menu_frame as *mut Widget);

        // Create the PackFiles List Context Menu Frame.
        let pack_files_list_context_menu_frame = GroupBox::new(&QString::from_std_str("Dependency Manager's Context Menu")).into_raw();
        let pack_files_list_context_menu_grid = create_grid_layout_unsafe(pack_files_list_context_menu_frame as *mut Widget);

        // Create the PackedFile Context Menu Frame.
        let packed_file_context_menu_frame = GroupBox::new(&QString::from_std_str("PackedFile's Context Menu")).into_raw();
        let packed_file_context_menu_grid = create_grid_layout_unsafe(packed_file_context_menu_frame as *mut Widget);

        // Create the DB Decoder Context Menu Frame.
        let db_decoder_context_menu_frame = GroupBox::new(&QString::from_std_str("DB Decoder's Context Menus")).into_raw();
        let db_decoder_context_menu_grid = create_grid_layout_unsafe(db_decoder_context_menu_frame as *mut Widget);

        //-------------------------------------------------------------------------------------------//
        // Creating the MenuBar's `PackFile` List...
        //-------------------------------------------------------------------------------------------//

        // Create the `PackFile` frame.
        let packfile_frame = GroupBox::new(&QString::from_std_str("PackFile")).into_raw();
        let packfile_grid = create_grid_layout_unsafe(packfile_frame as *mut Widget);

        // Create the `PackFile` list.
        let menu_bar_packfile_table = TableView::new().into_raw();
        let menu_bar_packfile_model = StandardItemModel::new(()).into_raw();
        unsafe { menu_bar_packfile_table.as_mut().unwrap().set_model(menu_bar_packfile_model as *mut AbstractItemModel); }

        // Disable sorting the columns and enlarge the last column.
        unsafe { menu_bar_packfile_table.as_mut().unwrap().set_sorting_enabled(false); }
        unsafe { menu_bar_packfile_table.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }

        // Add all the Lists to their respective grids.
        unsafe { packfile_grid.as_mut().unwrap().add_widget((menu_bar_packfile_table as *mut Widget, 0, 0, 1, 1)); }
        unsafe { menu_bar_grid.as_mut().unwrap().add_widget((packfile_frame as *mut Widget, 0, 0, 1, 1)); }

        //-------------------------------------------------------------------------------------------//
        // Creating the `About` List...
        //-------------------------------------------------------------------------------------------//

        // Create the `About` frame.
        let about_frame = GroupBox::new(&QString::from_std_str("About")).into_raw();
        let about_grid = create_grid_layout_unsafe(about_frame as *mut Widget);

        // Create the `PackFile` list.
        let menu_bar_about_table = TableView::new().into_raw();
        let menu_bar_about_model = StandardItemModel::new(()).into_raw();
        unsafe { menu_bar_about_table.as_mut().unwrap().set_model(menu_bar_about_model as *mut AbstractItemModel); }

        // Disable sorting the columns and enlarge the last column.
        unsafe { menu_bar_about_table.as_mut().unwrap().set_sorting_enabled(false); }
        unsafe { menu_bar_about_table.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }

        // Add all the Lists to their respective grids.
        unsafe { about_grid.as_mut().unwrap().add_widget((menu_bar_about_table as *mut Widget, 0, 0, 1, 1)); }
        unsafe { menu_bar_grid.as_mut().unwrap().add_widget((about_frame as *mut Widget, 0, 1, 1, 1)); }

        //-------------------------------------------------------------------------------------------//
        // Creating the Main TreeView Context Menu List...
        //-------------------------------------------------------------------------------------------//

        // Create the `Main TreeView Context Menu` list.
        let tree_view_context_menu_table = TableView::new().into_raw();
        let tree_view_context_menu_model = StandardItemModel::new(()).into_raw();
        unsafe { tree_view_context_menu_table.as_mut().unwrap().set_model(tree_view_context_menu_model as *mut AbstractItemModel); }

        // Disable sorting the columns and enlarge the last column.
        unsafe { tree_view_context_menu_table.as_mut().unwrap().set_sorting_enabled(false); }
        unsafe { tree_view_context_menu_table.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }

        // Add all the Lists to their respective grids.
        unsafe { tree_view_context_menu_grid.as_mut().unwrap().add_widget((tree_view_context_menu_table as *mut Widget, 0, 0, 1, 1)); }

        //-------------------------------------------------------------------------------------------//
        // Creating the PackFiles List Context Menu List...
        //-------------------------------------------------------------------------------------------//

        // Create the `PackFiles List Context Menu` list.
        let pack_files_list_context_menu_table = TableView::new().into_raw();
        let pack_files_list_context_menu_model = StandardItemModel::new(()).into_raw();
        unsafe { pack_files_list_context_menu_table.as_mut().unwrap().set_model(pack_files_list_context_menu_model as *mut AbstractItemModel); }

        // Disable sorting the columns and enlarge the last column.
        unsafe { pack_files_list_context_menu_table.as_mut().unwrap().set_sorting_enabled(false); }
        unsafe { pack_files_list_context_menu_table.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }

        // Add all the Lists to their respective grids.
        unsafe { pack_files_list_context_menu_grid.as_mut().unwrap().add_widget((pack_files_list_context_menu_table as *mut Widget, 0, 0, 1, 1)); }

        //-------------------------------------------------------------------------------------------//
        // Creating the PackedFile Table Context Menu List...
        //-------------------------------------------------------------------------------------------//

        // Create the `PackedFile Table` frame.
        let packed_files_table_frame = GroupBox::new(&QString::from_std_str("DB/Loc Table")).into_raw();
        let packed_files_table_grid = create_grid_layout_unsafe(packed_files_table_frame as *mut Widget);

        // Create the `Main TreeView Context Menu` list.
        let packed_files_table_context_menu_table = TableView::new().into_raw();
        let packed_files_table_context_menu_model = StandardItemModel::new(()).into_raw();
        unsafe { packed_files_table_context_menu_table.as_mut().unwrap().set_model(packed_files_table_context_menu_model as *mut AbstractItemModel); }

        // Disable sorting the columns and enlarge the last column.
        unsafe { packed_files_table_context_menu_table.as_mut().unwrap().set_sorting_enabled(false); }
        unsafe { packed_files_table_context_menu_table.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }

        // Add all the Lists to their respective grids.
        unsafe { packed_files_table_grid.as_mut().unwrap().add_widget((packed_files_table_context_menu_table as *mut Widget, 0, 0, 1, 1)); }
        unsafe { packed_file_context_menu_grid.as_mut().unwrap().add_widget((packed_files_table_frame as *mut Widget, 0, 0, 1, 1)); }

        //-------------------------------------------------------------------------------------------//
        // Creating the DB Decoder Field List Context Menu List...
        //-------------------------------------------------------------------------------------------//

        // Create the `Field List` frame.
        let fields_frame = GroupBox::new(&QString::from_std_str("Field List")).into_raw();
        let fields_grid = create_grid_layout_unsafe(fields_frame as *mut Widget);

        // Create the `Field's List` list.
        let fields_context_menu_table = TableView::new().into_raw();
        let fields_context_menu_model = StandardItemModel::new(()).into_raw();
        unsafe { fields_context_menu_table.as_mut().unwrap().set_model(fields_context_menu_model as *mut AbstractItemModel); }

        // Disable sorting the columns and enlarge the last column.
        unsafe { fields_context_menu_table.as_mut().unwrap().set_sorting_enabled(false); }
        unsafe { fields_context_menu_table.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }

        // Add all the Lists to their respective grids.
        unsafe { fields_grid.as_mut().unwrap().add_widget((fields_context_menu_table as *mut Widget, 0, 0, 1, 1)); }
        unsafe { db_decoder_context_menu_grid.as_mut().unwrap().add_widget((fields_frame as *mut Widget, 0, 0, 1, 1)); }

        //-------------------------------------------------------------------------------------------//
        // Creating the DB Decoder Version List Context Menu List...
        //-------------------------------------------------------------------------------------------//

        // Create the `Version List` frame.
        let versions_frame = GroupBox::new(&QString::from_std_str("Version List")).into_raw();
        let versions_grid = create_grid_layout_unsafe(versions_frame as *mut Widget);

        // Create the `Version's List` list.
        let versions_context_menu_table = TableView::new().into_raw();
        let versions_context_menu_model = StandardItemModel::new(()).into_raw();
        unsafe { versions_context_menu_table.as_mut().unwrap().set_model(versions_context_menu_model as *mut AbstractItemModel); }

        // Disable sorting the columns and enlarge the last column.
        unsafe { versions_context_menu_table.as_mut().unwrap().set_sorting_enabled(false); }
        unsafe { versions_context_menu_table.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }

        // Add all the Lists to their respective grids.
        unsafe { versions_grid.as_mut().unwrap().add_widget((versions_context_menu_table as *mut Widget, 0, 0, 1, 1)); }
        unsafe { db_decoder_context_menu_grid.as_mut().unwrap().add_widget((versions_frame as *mut Widget, 0, 1, 1, 1)); }

        //-------------------------------------------------------------------------------------------//
        // Adding all the frames to the main grid...
        //-------------------------------------------------------------------------------------------//

        // Add everything to the Window.
        unsafe { main_grid.as_mut().unwrap().add_widget((menu_bar_frame as *mut Widget, 0, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((tree_view_context_menu_frame as *mut Widget, 0, 1, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((pack_files_list_context_menu_frame as *mut Widget, 0, 2, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((packed_file_context_menu_frame as *mut Widget, 1, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((db_decoder_context_menu_frame as *mut Widget, 1, 1, 1, 2)); }

        // Create the bottom ButtonBox.
        let mut button_box = DialogButtonBox::new(());
        unsafe { main_grid.as_mut().unwrap().add_widget((button_box.static_cast_mut() as *mut Widget, 2, 0, 1, 3)); }

        // Create the bottom Buttons.
        let restore_default_button;
        let cancel_button;
        let accept_button;

        // Add them to the Dialog.
        restore_default_button = button_box.add_button(dialog_button_box::StandardButton::RestoreDefaults);
        cancel_button = button_box.add_button(dialog_button_box::StandardButton::Cancel);
        accept_button = button_box.add_button(dialog_button_box::StandardButton::Save);

        //-------------------------------------------------------------------------------------------//
        // Put all the important things together...
        //-------------------------------------------------------------------------------------------//

        let mut shortcuts_dialog = Self {
            menu_bar_packfile: menu_bar_packfile_model,
            menu_bar_about: menu_bar_about_model,
            tree_view: tree_view_context_menu_model,
            pack_files_list: pack_files_list_context_menu_model,
            packed_files_table: packed_files_table_context_menu_model,
            db_decoder_fields: fields_context_menu_model,
            db_decoder_definitions: versions_context_menu_model,
        };

        //-------------------------------------------------------------------------------------------//
        // Loading data to the Settings Dialog...
        //-------------------------------------------------------------------------------------------//

        // Load the MyMod Path, if exists.
        shortcuts_dialog.load_to_shortcuts_dialog(&SHORTCUTS.lock().unwrap());

        //-------------------------------------------------------------------------------------------//
        // Slots and stuff...
        //-------------------------------------------------------------------------------------------//

        let shortcuts_dialog = Rc::new(RefCell::new(shortcuts_dialog));

        // What happens when we hit the "Restore Default" action.
        let slot_restore_default = SlotNoArgs::new(clone!(
            shortcuts_dialog => move || {
                (*shortcuts_dialog.borrow_mut()).load_to_shortcuts_dialog(&Shortcuts::new())
            }
        ));

        // What happens when we hit the "Restore Default" button.
        unsafe { restore_default_button.as_mut().unwrap().signals().released().connect(&slot_restore_default); }

        // What happens when we hit the "Cancel" button.
        unsafe { cancel_button.as_mut().unwrap().signals().released().connect(&dialog.as_mut().unwrap().slots().close()); }

        // What happens when we hit the "Accept" button.
        unsafe { accept_button.as_mut().unwrap().signals().released().connect(&dialog.as_mut().unwrap().slots().accept()); }

        // Show the Dialog, save the current shortcuts, and return them.
        unsafe { if dialog.as_mut().unwrap().exec() == 1 { Some(shortcuts_dialog.borrow().save_from_shortcuts_dialog()) }

        // Otherwise, return None.
        else { None } }
    }

    /// This function loads the data from the Shortcuts struct to the Shortcuts Dialog.
    pub fn load_to_shortcuts_dialog(&mut self, shortcuts: &Shortcuts) {

        // Clear all the models, just in case this is a restore default operation.
        unsafe { self.menu_bar_packfile.as_mut().unwrap().clear(); }
        unsafe { self.menu_bar_about.as_mut().unwrap().clear(); }
        unsafe { self.tree_view.as_mut().unwrap().clear(); }
        unsafe { self.pack_files_list.as_mut().unwrap().clear(); }
        unsafe { self.packed_files_table.as_mut().unwrap().clear(); }
        unsafe { self.db_decoder_fields.as_mut().unwrap().clear(); }
        unsafe { self.db_decoder_definitions.as_mut().unwrap().clear(); }

        // Just add in mass the shortcuts to the Models.
        for (key, value) in shortcuts.menu_bar_packfile.iter() {
            let mut row_list = ListStandardItemMutPtr::new(());
            unsafe { row_list.append_unsafe(&StandardItem::new(&QString::from_std_str(key)).into_raw()); }
            unsafe { row_list.append_unsafe(&StandardItem::new(&QString::from_std_str(value)).into_raw()); }
            unsafe { row_list.at(0).as_mut().unwrap().set_editable(false); }
            unsafe { self.menu_bar_packfile.as_mut().unwrap().append_row(&row_list); }
        }

        for (key, value) in shortcuts.menu_bar_about.iter() {
            let mut row_list = ListStandardItemMutPtr::new(());
            unsafe { row_list.append_unsafe(&StandardItem::new(&QString::from_std_str(key)).into_raw()); }
            unsafe { row_list.append_unsafe(&StandardItem::new(&QString::from_std_str(value)).into_raw()); }
            unsafe { row_list.at(0).as_mut().unwrap().set_editable(false); }
            unsafe { self.menu_bar_about.as_mut().unwrap().append_row(&row_list); }
        }

        for (key, value) in shortcuts.tree_view.iter() {
            let mut row_list = ListStandardItemMutPtr::new(());
            unsafe { row_list.append_unsafe(&StandardItem::new(&QString::from_std_str(key)).into_raw()); }
            unsafe { row_list.append_unsafe(&StandardItem::new(&QString::from_std_str(value)).into_raw()); }
            unsafe { row_list.at(0).as_mut().unwrap().set_editable(false); }
            unsafe { self.tree_view.as_mut().unwrap().append_row(&row_list); }
        }

        for (key, value) in shortcuts.pack_files_list.iter() {
            let mut row_list = ListStandardItemMutPtr::new(());
            unsafe { row_list.append_unsafe(&StandardItem::new(&QString::from_std_str(key)).into_raw()); }
            unsafe { row_list.append_unsafe(&StandardItem::new(&QString::from_std_str(value)).into_raw()); }
            unsafe { row_list.at(0).as_mut().unwrap().set_editable(false); }
            unsafe { self.pack_files_list.as_mut().unwrap().append_row(&row_list); }
        }

        for (key, value) in shortcuts.packed_files_table.iter() {
            let mut row_list = ListStandardItemMutPtr::new(());
            unsafe { row_list.append_unsafe(&StandardItem::new(&QString::from_std_str(key)).into_raw()); }
            unsafe { row_list.append_unsafe(&StandardItem::new(&QString::from_std_str(value)).into_raw()); }
            unsafe { row_list.at(0).as_mut().unwrap().set_editable(false); }
            unsafe { self.packed_files_table.as_mut().unwrap().append_row(&row_list); }
        }

        for (key, value) in shortcuts.db_decoder_fields.iter() {
            let mut row_list = ListStandardItemMutPtr::new(());
            unsafe { row_list.append_unsafe(&StandardItem::new(&QString::from_std_str(key)).into_raw()); }
            unsafe { row_list.append_unsafe(&StandardItem::new(&QString::from_std_str(value)).into_raw()); }
            unsafe { row_list.at(0).as_mut().unwrap().set_editable(false); }
            unsafe { self.db_decoder_fields.as_mut().unwrap().append_row(&row_list); }
        }

        for (key, value) in shortcuts.db_decoder_definitions.iter() {
            let mut row_list = ListStandardItemMutPtr::new(());
            unsafe { row_list.append_unsafe(&StandardItem::new(&QString::from_std_str(key)).into_raw()); }
            unsafe { row_list.append_unsafe(&StandardItem::new(&QString::from_std_str(value)).into_raw()); }
            unsafe { row_list.at(0).as_mut().unwrap().set_editable(false); }
            unsafe { self.db_decoder_definitions.as_mut().unwrap().append_row(&row_list); }
        }

        // Rename the columns.
        unsafe { self.menu_bar_packfile.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Action")))); }
        unsafe { self.menu_bar_packfile.as_mut().unwrap().set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Shortcut")))); }

        unsafe { self.menu_bar_about.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Action")))); }
        unsafe { self.menu_bar_about.as_mut().unwrap().set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Shortcut")))); }
        
        unsafe { self.tree_view.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Action")))); }
        unsafe { self.tree_view.as_mut().unwrap().set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Shortcut")))); }
        
        unsafe { self.pack_files_list.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Action")))); }
        unsafe { self.pack_files_list.as_mut().unwrap().set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Shortcut")))); }
        
        unsafe { self.packed_files_table.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Action")))); }
        unsafe { self.packed_files_table.as_mut().unwrap().set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Shortcut")))); }

        unsafe { self.db_decoder_fields.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Action")))); }
        unsafe { self.db_decoder_fields.as_mut().unwrap().set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Shortcut")))); }
        
        unsafe { self.db_decoder_definitions.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Action")))); }
        unsafe { self.db_decoder_definitions.as_mut().unwrap().set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Shortcut")))); }
    }

    /// This function gets the data from the `ShortcutsDialog` and returns a `Shortcuts` struct with that data in it.
    pub fn save_from_shortcuts_dialog(&self) -> Shortcuts {

        // Create a new Shortcuts struct to populate it.
        let mut shortcuts = Shortcuts::new();

        // Get the amount of rows we have.
        let menu_bar_packfile_rows;
        unsafe { menu_bar_packfile_rows = self.menu_bar_packfile.as_mut().unwrap().row_count(()); }

        // For each row we have...
        for row in 0..menu_bar_packfile_rows {

            // Make a new entry with the data from the `ListStore`, and push it to our new `LocData`.
            unsafe { shortcuts.menu_bar_packfile.insert(
                QString::to_std_string(&self.menu_bar_packfile.as_mut().unwrap().item((row as i32, 0)).as_mut().unwrap().text()),
                QString::to_std_string(&self.menu_bar_packfile.as_mut().unwrap().item((row as i32, 1)).as_mut().unwrap().text())
            ); }
        }

        // And rinse and repeat.
        let menu_bar_about_rows;
        unsafe { menu_bar_about_rows = self.menu_bar_about.as_mut().unwrap().row_count(()); }
        for row in 0..menu_bar_about_rows {
            unsafe { shortcuts.menu_bar_about.insert(
                QString::to_std_string(&self.menu_bar_about.as_mut().unwrap().item((row as i32, 0)).as_mut().unwrap().text()),
                QString::to_std_string(&self.menu_bar_about.as_mut().unwrap().item((row as i32, 1)).as_mut().unwrap().text())
            ); }
        }

        let tree_view_rows;
        unsafe { tree_view_rows = self.tree_view.as_mut().unwrap().row_count(()); }
        for row in 0..tree_view_rows {
            unsafe { shortcuts.tree_view.insert(
                QString::to_std_string(&self.tree_view.as_mut().unwrap().item((row as i32, 0)).as_mut().unwrap().text()),
                QString::to_std_string(&self.tree_view.as_mut().unwrap().item((row as i32, 1)).as_mut().unwrap().text())
            ); }
        }

        let pack_files_list_rows;
        unsafe { pack_files_list_rows = self.pack_files_list.as_mut().unwrap().row_count(()); }
        for row in 0..pack_files_list_rows {
            unsafe { shortcuts.pack_files_list.insert(
                QString::to_std_string(&self.pack_files_list.as_mut().unwrap().item((row as i32, 0)).as_mut().unwrap().text()),
                QString::to_std_string(&self.pack_files_list.as_mut().unwrap().item((row as i32, 1)).as_mut().unwrap().text())
            ); }
        }

        let packed_file_table_rows;
        unsafe { packed_file_table_rows = self.packed_files_table.as_mut().unwrap().row_count(()); }
        for row in 0..packed_file_table_rows {
            unsafe { shortcuts.packed_files_table.insert(
                QString::to_std_string(&self.packed_files_table.as_mut().unwrap().item((row as i32, 0)).as_mut().unwrap().text()),
                QString::to_std_string(&self.packed_files_table.as_mut().unwrap().item((row as i32, 1)).as_mut().unwrap().text())
            ); }
        }

        let db_decoder_fields_rows;
        unsafe { db_decoder_fields_rows = self.db_decoder_fields.as_mut().unwrap().row_count(()); }
        for row in 0..db_decoder_fields_rows {
            unsafe { shortcuts.db_decoder_fields.insert(
                QString::to_std_string(&self.db_decoder_fields.as_mut().unwrap().item((row as i32, 0)).as_mut().unwrap().text()),
                QString::to_std_string(&self.db_decoder_fields.as_mut().unwrap().item((row as i32, 1)).as_mut().unwrap().text())
            ); }
        }

        let db_decoder_definitions_rows;
        unsafe { db_decoder_definitions_rows = self.db_decoder_definitions.as_mut().unwrap().row_count(()); }
        for row in 0..db_decoder_definitions_rows {
            unsafe { shortcuts.db_decoder_definitions.insert(
                QString::to_std_string(&self.db_decoder_definitions.as_mut().unwrap().item((row as i32, 0)).as_mut().unwrap().text()),
                QString::to_std_string(&self.db_decoder_definitions.as_mut().unwrap().item((row as i32, 1)).as_mut().unwrap().text())
            ); }
        }

        // Return the new Shortcuts.
        shortcuts
    }
}
