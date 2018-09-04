// In this file is all the stuff needed for the dependency manager to work.
extern crate qt_widgets;
extern crate qt_gui;
extern crate qt_core;

use qt_widgets::menu::Menu;
use qt_widgets::slots::SlotQtCorePointRef;
use qt_widgets::table_view::TableView;
use qt_widgets::widget::Widget;

use qt_gui::brush::Brush;
use qt_gui::cursor::Cursor;
use qt_gui::gui_application::GuiApplication;
use qt_gui::key_sequence::KeySequence;
use qt_gui::list::ListStandardItemMutPtr;
use qt_gui::slots::SlotStandardItemMutPtr;
use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::connection::Signal;
use qt_core::variant::Variant;
use qt_core::slots::{SlotBool, SlotItemSelectionRefItemSelectionRef, SlotModelIndexRefModelIndexRefVectorVectorCIntRef};

use qt_core::qt::{Orientation, ContextMenuPolicy, ShortcutContext, GlobalColor};

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver};

use AppUI;
use Commands;
use Data;
use QString;
use common::*;
use common::communications::*;
use ui::*;

/// Struct `DependencyTableView`: contains all the stuff we need to give to the program to show a
/// `TableView` with the dependency list of the PackFile, allowing us to manipulate it.
pub struct DependencyTableView {
    pub slot_context_menu: SlotQtCorePointRef<'static>,
    pub slot_context_menu_enabler: SlotItemSelectionRefItemSelectionRef<'static>,
    pub save_changes: SlotModelIndexRefModelIndexRefVectorVectorCIntRef<'static>,
    pub slot_item_changed: SlotStandardItemMutPtr<'static>,
    pub slot_context_menu_add: SlotBool<'static>,
    pub slot_context_menu_insert: SlotBool<'static>,
    pub slot_context_menu_delete: SlotBool<'static>,
    pub slot_context_menu_copy: SlotBool<'static>,
    pub slot_context_menu_paste: SlotBool<'static>,
    pub slot_context_menu_paste_as_new_lines: SlotBool<'static>,
}

/// Implementation of DependencyTableView.
impl DependencyTableView {

    /// This functin returns a dummy struct. Use it for initialization.
    pub fn new() -> Self {

        // Create some dummy slots and return it.
        Self {
            slot_context_menu: SlotQtCorePointRef::new(|_| {}),
            slot_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(|_,_| {}),
            save_changes: SlotModelIndexRefModelIndexRefVectorVectorCIntRef::new(|_,_,_| {}),
            slot_item_changed: SlotStandardItemMutPtr::new(|_| {}),
            slot_context_menu_add: SlotBool::new(|_| {}),
            slot_context_menu_insert: SlotBool::new(|_| {}),
            slot_context_menu_delete: SlotBool::new(|_| {}),
            slot_context_menu_copy: SlotBool::new(|_| {}),
            slot_context_menu_paste: SlotBool::new(|_| {}),
            slot_context_menu_paste_as_new_lines: SlotBool::new(|_| {}),
        }
    }

    /// This function creates a new TableView with the PackedFile's View as father and returns a
    /// `DependencyTableView` with all his data.
    pub fn create_table_view(
        sender_qt: Sender<Commands>,
        sender_qt_data: &Sender<Data>,
        receiver_qt: &Rc<RefCell<Receiver<Data>>>,
        is_modified: &Rc<RefCell<bool>>,
        app_ui: &AppUI,
    ) -> Self {

        // Send the index back to the background thread, and wait until we get a response.
        sender_qt.send(Commands::GetPackFilesList).unwrap();
        let pack_files = if let Data::VecString(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

        // Create the TableView.
        let table_view = TableView::new().into_raw();
        let model = StandardItemModel::new(()).into_raw();

        // Make the last column fill all the available space.
        unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }

        // Prepare the TableView to have a Contextual Menu.
        unsafe { table_view.as_mut().unwrap().set_context_menu_policy(ContextMenuPolicy::Custom); }

        // Disable sorting the columns, as we don't know exactly if the order affects how it gets loaded.
        unsafe { table_view.as_mut().unwrap().set_sorting_enabled(false); }

        // Load the data to the Table. For some reason, if we do this after setting the titles of
        // the columns, the titles will be reseted to 1, 2, 3,... so we do this here.
        Self::load_data_to_table_view(&pack_files, model);

        // Make both headers visible.
        unsafe { table_view.as_mut().unwrap().vertical_header().as_mut().unwrap().set_visible(true); }
        unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_visible(true); }

        // Add Table to the Grid.
        unsafe { table_view.as_mut().unwrap().set_model(model as *mut AbstractItemModel); }
        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((table_view as *mut Widget, 0, 0, 1, 1)); }

        // Set the title of the column.
        unsafe { model.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("PackFiles List")))); }

        // Create the Contextual Menu for the TableView.
        let mut context_menu = Menu::new(());
        let context_menu_add = context_menu.add_action(&QString::from_std_str("&Add Row"));
        let context_menu_insert = context_menu.add_action(&QString::from_std_str("&Insert Row"));
        let context_menu_delete = context_menu.add_action(&QString::from_std_str("&Delete Row"));
        let context_menu_copy = context_menu.add_action(&QString::from_std_str("&Copy"));
        let mut context_menu_paste_submenu = Menu::new(&QString::from_std_str("&Paste..."));
        let context_menu_paste = context_menu_paste_submenu.add_action(&QString::from_std_str("&Paste in Selection"));
        let context_menu_paste_as_new_lines = context_menu_paste_submenu.add_action(&QString::from_std_str("&Paste as New Rows"));

        // Get the current shortcuts.
        sender_qt.send(Commands::GetShortcuts).unwrap();
        let shortcuts = if let Data::Shortcuts(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

        // Set the shortcuts for these actions.
        unsafe { context_menu_add.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.pack_files_list.get("add_row").unwrap()))); }
        unsafe { context_menu_insert.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.pack_files_list.get("insert_row").unwrap()))); }
        unsafe { context_menu_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.pack_files_list.get("delete_row").unwrap()))); }
        unsafe { context_menu_copy.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.pack_files_list.get("copy").unwrap()))); }
        unsafe { context_menu_paste.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.pack_files_list.get("paste").unwrap()))); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.pack_files_list.get("paste_as_new_row").unwrap()))); }

        // Set the shortcuts to only trigger in the Table.
        unsafe { context_menu_add.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_insert.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_copy.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_paste.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }

        // Add the actions to the TableView, so the shortcuts work.
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_add); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_insert); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_delete); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_copy); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_paste); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_paste_as_new_lines); }

        // Status Tips for the actions.
        unsafe { context_menu_add.as_mut().unwrap().set_status_tip(&QString::from_std_str("Add an empty cell to the list.")); }
        unsafe { context_menu_insert.as_mut().unwrap().set_status_tip(&QString::from_std_str("Insert an empty cell just above the one selected.")); }
        unsafe { context_menu_delete.as_mut().unwrap().set_status_tip(&QString::from_std_str("Delete all the selected cells.")); }
        unsafe { context_menu_copy.as_mut().unwrap().set_status_tip(&QString::from_std_str("Copy whatever is selected to the Clipboard.")); }
        unsafe { context_menu_paste.as_mut().unwrap().set_status_tip(&QString::from_std_str("Try to paste whatever is in the Clipboard. Does nothing if the data is not compatible with the cell.")); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().set_status_tip(&QString::from_std_str("Try to paste whatever is in the Clipboard as new lines at the end of the list. Does nothing if the data is not compatible with the cell.")); }

        // Insert some separators to space the menu, and the paste submenu.
        unsafe { context_menu.insert_separator(context_menu_copy); }
        unsafe { context_menu.add_menu_unsafe(context_menu_paste_submenu.into_raw()); }

        // Slots for the TableView...
        let slots = Self {
            slot_context_menu: SlotQtCorePointRef::new(move |_| { context_menu.exec2(&Cursor::pos()); }),
            slot_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(move  |_,_| {

                    // Turns out that this slot doesn't give the the amount of selected items, so we have to get them ourselfs.
                    let selection_model;
                    let selection;
                    unsafe { selection_model = table_view.as_mut().unwrap().selection_model(); }
                    unsafe { selection = selection_model.as_mut().unwrap().selected_indexes(); }

                    // If we have something selected, enable these actions.
                    if selection.count(()) > 0 {
                        unsafe {
                            context_menu_copy.as_mut().unwrap().set_enabled(true);
                            context_menu_delete.as_mut().unwrap().set_enabled(true);
                        }
                    }

                    // Otherwise, disable them.
                    else {
                        unsafe {
                            context_menu_copy.as_mut().unwrap().set_enabled(false);
                            context_menu_delete.as_mut().unwrap().set_enabled(false);
                        }
                    }
                }
            ),
            save_changes: SlotModelIndexRefModelIndexRefVectorVectorCIntRef::new(clone!(
                app_ui,
                is_modified,
                sender_qt,
                sender_qt_data => move |_,_,roles| {

                    // To avoid doing this multiple times due to the cell painting stuff, we need to check the role.
                    // This has to be allowed ONLY if the role is 0 (DisplayText) or 2 (EditorText).
                    if roles.contains(&0) || roles.contains(&2) {                    

                        // Check for errors.
                        Self::check_errors(model);
                        
                        // Get the new LocData to send.
                        let list = Self::return_data_from_table_view(model);

                        // Tell the background thread to start saving the PackedFile.
                        sender_qt.send(Commands::SetPackFilesList).unwrap();
                        sender_qt_data.send(Data::VecString(list)).unwrap();

                        // Set the mod as "Modified".
                        unsafe { *is_modified.borrow_mut() = set_modified(true, &app_ui, Some(vec![app_ui.folder_tree_model.as_ref().unwrap().item(0).as_ref().unwrap().text().to_std_string()])); }
                    }
                }
            )),
            slot_item_changed: SlotStandardItemMutPtr::new(|item| {
                unsafe { item.as_mut().unwrap().set_background(&Brush::new(GlobalColor::Yellow)); }
            }),

            slot_context_menu_add: SlotBool::new(move |_| {

                // Create a new list of StandardItem.
                let mut qlist = ListStandardItemMutPtr::new(());

                // Create an empty row.
                let mut key = StandardItem::new(&QString::from_std_str(""));

                // Paint the cells.
                key.set_background(&Brush::new(GlobalColor::Green));

                // Add an empty row to the list.
                unsafe { qlist.append_unsafe(&key.into_raw()); }

                // Append the new row.
                unsafe { model.as_mut().unwrap().append_row(&qlist); }
            }),
            slot_context_menu_insert: SlotBool::new(move |_| {

                // Create a new list of StandardItem.
                let mut qlist = ListStandardItemMutPtr::new(());

                // Create an empty row.
                let mut key = StandardItem::new(&QString::from_std_str(""));

                // Paint the cells.
                key.set_background(&Brush::new(GlobalColor::Green));

                // Add an empty row to the list.
                unsafe { qlist.append_unsafe(&key.into_raw()); }

                // Get the current row.
                let selection;
                unsafe { selection = table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }

                // If there is any row selected...
                if selection.indexes().count(()) > 0 {

                    // Get the current filtered ModelIndex.
                    let model_index_list = selection.indexes();
                    let model_index = model_index_list.at(0);

                    // Check if the ModelIndex is valid. Otherwise this can crash.
                    if model_index.is_valid() {

                        // Get the current row.
                        let row = model_index.row();

                        // Insert the new row where the current one is.
                        unsafe { model.as_mut().unwrap().insert_row((row, &qlist)); }
                    }
                }

                // Otherwise, just do the same the "Add Row" do.
                else { unsafe { model.as_mut().unwrap().append_row(&qlist); } }
            }),
            slot_context_menu_delete: SlotBool::new(clone!(
                app_ui,
                is_modified,
                sender_qt,
                sender_qt_data => move |_| {

                    // Get the current selection.
                    let selection;
                    unsafe { selection = table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                    let indexes = selection.indexes();

                    // Get all the selected rows.
                    let mut rows: Vec<i32> = vec![];
                    for index in 0..indexes.size() {

                        // Get the ModelIndex.
                        let model_index = indexes.at(index);

                        // Check if the ModelIndex is valid. Otherwise this can crash.
                        if model_index.is_valid() {

                            // Get the current row.
                            let row = model_index.row();

                            // Add it to the list.
                            rows.push(row);
                        }
                    }

                    // Sort the list and reverse it.
                    rows.sort();
                    rows.reverse();

                    // Delete every selected row. '_y' is ignorable.
                    let mut _y = false;
                    unsafe { rows.iter().for_each(|x| _y = model.as_mut().unwrap().remove_rows((*x, 1))); }

                    // If we deleted anything, save the data.
                    if rows.len() > 0 {

                        // Get the new LocData to send.
	                    let list = Self::return_data_from_table_view(model);

	                    // Tell the background thread to start saving the PackedFile.
	                    sender_qt.send(Commands::SetPackFilesList).unwrap();
	                    sender_qt_data.send(Data::VecString(list)).unwrap();

	                    // Set the mod as "Modified".
	                    *is_modified.borrow_mut() = set_modified(true, &app_ui, None);
                    }
                }
            )),
            slot_context_menu_copy: SlotBool::new(move |_| {

                // Create a string to keep all the values in a TSV format (x\tx\tx).
                let mut copy = String::new();

                // Get the current selection.
                let selection;
                unsafe { selection = table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                let indexes = selection.indexes();

                // Create a variable to check the row of the model_index.
                let mut row = 0;

                // For each selected index...
                for (cycle, index) in (0..indexes.count(())).enumerate() {

                    // Get his ModelIndex.
                    let model_index = indexes.at(index);

                    // Check if the ModelIndex is valid. Otherwise this can crash.
                    if model_index.is_valid() {

                        // Get his StandardItem.
                        let standard_item;
                        unsafe { standard_item = model.as_mut().unwrap().item_from_index(&model_index); }

                        // If this is the first time we loop, get the row.
                        if cycle == 0 { row = model_index.row(); }

                        // Otherwise, if our current row is different than our last row...
                        else if model_index.row() != row {

                            // Replace the last \t with a \n
                            copy.pop();
                            copy.push('\n');

                            // Update the row.
                            row = model_index.row();
                        }

                        // Get his text and push them to the copy string.
                        unsafe { copy.push_str(&QString::to_std_string(&standard_item.as_mut().unwrap().text())); }

                        // Add a \t to separate fields except if it's the last field.
                        if index < (indexes.count(()) - 1) { copy.push('\t'); }
                    }
                }

                // Put the baby into the oven.
                unsafe { GuiApplication::clipboard().as_mut().unwrap().set_text(&QString::from_std_str(copy)); }
            }),
            slot_context_menu_paste: SlotBool::new(clone!(
                app_ui,
                is_modified,
                sender_qt,
                sender_qt_data => move |_| {

                    // Get the clipboard.
                    let clipboard = GuiApplication::clipboard();

                    // Get the current selection.
                    let selection;
                    unsafe { selection = table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                    let indexes = selection.indexes();

                    // Get the text from the clipboard.
                    let mut text;
                    unsafe { text = QString::to_std_string(&clipboard.as_mut().unwrap().text(())); }

                    // If the text ends in \n, remove it. Excel things.
                    if text.ends_with('\n') { text.pop(); }

                    // We don't use newlines, so replace them with '\t'.
                    let text = text.replace('\n', "\t");

                    // Split the text into individual strings.
                    let text = text.split('\t').collect::<Vec<&str>>();

                    // Vector to store the selected items.
                    let mut items = vec![];

                    // For each selected index...
                    for index in 0..indexes.count(()) {

                        // Get the filtered ModelIndex.
                        let model_index = indexes.at(index);

                        // Check if the ModelIndex is valid. Otherwise this can crash.
                        if model_index.is_valid() {

                            // Get his StandardItem and add it to the Vector.
                            unsafe { items.push(model.as_mut().unwrap().item_from_index(&model_index)); }
                        }
                    }

                    // Zip together both vectors.
                    let data = items.iter().zip(text);

                    // For each cell we have...
                    for cell in data.clone() {

                        // Otherwise, it's just a string.
                        unsafe { cell.0.as_mut().unwrap().set_text(&QString::from_std_str(cell.1)); }

                        // Paint the cells.
                        unsafe { cell.0.as_mut().unwrap().set_background(&Brush::new(GlobalColor::Yellow)); }
                    }

                    // If we pasted anything, save.
                    if data.count() > 0 {

                        // Get the new LocData to send.
	                    let list = Self::return_data_from_table_view(model);

	                    // Tell the background thread to start saving the PackedFile.
	                    sender_qt.send(Commands::SetPackFilesList).unwrap();
	                    sender_qt_data.send(Data::VecString(list)).unwrap();

	                    // Set the mod as "Modified".
	                    *is_modified.borrow_mut() = set_modified(true, &app_ui, None);
                    }
                }
            )),

            slot_context_menu_paste_as_new_lines: SlotBool::new(clone!(
                app_ui,
                is_modified,
                sender_qt,
                sender_qt_data => move |_| {

                    // Get the clipboard.
                    let clipboard = GuiApplication::clipboard();

                    // Get the text from the clipboard.
                    let mut text;
                    unsafe { text = QString::to_std_string(&clipboard.as_mut().unwrap().text(())); }

                    // If the text ends in \n, remove it. Excel things.
                    if text.ends_with('\n') { text.pop(); }

                    // We don't use newlines, so replace them with '\t'.
                    let text = text.replace('\n', "\t");

                    // Split the text into individual strings.
                    let text = text.split('\t').collect::<Vec<&str>>();

                    // For each text we have to paste...
                    for cell in &text {

	                    // Create a new list of StandardItem.
	                    let mut qlist = ListStandardItemMutPtr::new(());

                        // Create the item to add to the row.
                        let mut item = StandardItem::new(());

                        // Prepare the item.
                        item.set_text(&QString::from_std_str(cell));
                        item.set_background(&Brush::new(GlobalColor::Green));
                        
                        // Add the item to the list.
                        unsafe { qlist.append_unsafe(&item.into_raw()); }

                        // Append the list to the Table.
                        unsafe { model.as_mut().unwrap().append_row(&qlist); }
                    }

                    // If we pasted anything, save.
                    if !text.is_empty() {

                        // Get the new LocData to send.
	                    let list = Self::return_data_from_table_view(model);

	                    // Tell the background thread to start saving the PackedFile.
	                    sender_qt.send(Commands::SetPackFilesList).unwrap();
	                    sender_qt_data.send(Data::VecString(list)).unwrap();

	                    // Set the mod as "Modified".
	                    *is_modified.borrow_mut() = set_modified(true, &app_ui, None);
                    }
                }
            )),
        };

        // Actions for the TableView...
        unsafe { (table_view as *mut Widget).as_ref().unwrap().signals().custom_context_menu_requested().connect(&slots.slot_context_menu); }
        unsafe { model.as_mut().unwrap().signals().data_changed().connect(&slots.save_changes); }
        unsafe { model.as_mut().unwrap().signals().item_changed().connect(&slots.slot_item_changed); }
        unsafe { context_menu_add.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_add); }
        unsafe { context_menu_insert.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_insert); }
        unsafe { context_menu_delete.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_delete); }
        unsafe { context_menu_copy.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_copy); }
        unsafe { context_menu_paste.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_paste); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_paste_as_new_lines); }

        // Initial states for the Contextual Menu Actions.
        unsafe {
            context_menu_add.as_mut().unwrap().set_enabled(true);
            context_menu_insert.as_mut().unwrap().set_enabled(true);
            context_menu_delete.as_mut().unwrap().set_enabled(false);
            context_menu_copy.as_mut().unwrap().set_enabled(false);
            context_menu_paste.as_mut().unwrap().set_enabled(true);
            context_menu_paste_as_new_lines.as_mut().unwrap().set_enabled(true);
        }

        // Trigger the "Enable/Disable" slot every time we change the selection in the TreeView.
        unsafe { table_view.as_mut().unwrap().selection_model().as_ref().unwrap().signals().selection_changed().connect(&slots.slot_context_menu_enabler); }

        // Return the slots to keep them as hostages.
        return slots
    }

    /// This function loads the data from a list into a TableView.
    pub fn load_data_to_table_view(
        data: &[String],
        model: *mut StandardItemModel,
    ) {
        // First, we delete all the data from the model. Just in case there is something there.
        unsafe { model.as_mut().unwrap().clear(); }

        // Then we add every line to the model.
        for packfile in data {

            // Create a new list of StandardItem.
            let mut qlist = ListStandardItemMutPtr::new(());

            // Create the items of the new row.
            let key = StandardItem::new(&QString::from_std_str(&packfile));

            // Add the items to the list.
            unsafe { qlist.append_unsafe(&key.into_raw()); }

            // Just append a new row.
            unsafe { model.as_mut().unwrap().append_row(&qlist); }
        }

        // If there are no entries, add an empty row with default values, so Qt shows the table anyway.
        if data.len() == 0 {

            // Create a new list of StandardItem.
            let mut qlist = ListStandardItemMutPtr::new(());

            // Create the items of the new row.
            let key = StandardItem::new(&QString::from_std_str(""));

            // Add the items to the list.
            unsafe { qlist.append_unsafe(&key.into_raw()); }

            // Just append a new row.
            unsafe { model.as_mut().unwrap().append_row(&qlist); }

            // Remove the row, so the columns stay.
            unsafe { model.as_mut().unwrap().remove_rows((0, 1)); }
        }

        // Check for errors.
        Self::check_errors(model);
    }

    /// This function returns a Vec<String> with all the stuff in the table.
    pub fn return_data_from_table_view(
        model: *mut StandardItemModel,
    ) -> Vec<String> {

        let mut data = vec![];
        let rows;
        unsafe { rows = model.as_mut().unwrap().row_count(()); }

        for row in 0..rows {
            unsafe { data.push(QString::to_std_string(&model.as_mut().unwrap().item((row as i32, 0)).as_mut().unwrap().text())); }
        }

        data
    }

    /// This function checks if the PackFiles in the model are valid, and paints as red the invalid ones.
    pub fn check_errors( model: *mut StandardItemModel) {

        // For each row...
        let rows;
        unsafe { rows = model.as_mut().unwrap().row_count(()); }
        for row in 0..rows {

            // Get the item on the row.
            let item;
            unsafe { item = model.as_mut().unwrap().item((row as i32, 0)); }

            // Get the PackFile's name.
            let packfile;
            unsafe { packfile = item.as_mut().unwrap().text().to_std_string(); }

            // We paint it depending on if it's a valid PackFile or not.
            if !packfile.is_empty() && packfile.ends_with(".pack") && !packfile.contains(' ') { unsafe { item.as_mut().unwrap().set_foreground(&Brush::new(GlobalColor::Black)); } }
            else { unsafe { item.as_mut().unwrap().set_foreground(&Brush::new(GlobalColor::Red)); } }
        }  
    }
}
