// In this file are all the helper functions used by the UI (mainly Qt here)
extern crate chrono;
extern crate qt_widgets;
extern crate qt_gui;
extern crate qt_core;
extern crate cpp_utils;
extern crate serde_json;

use qt_widgets::abstract_button::AbstractButton;
use qt_widgets::action::Action;
use qt_widgets::button_group::ButtonGroup;
use qt_widgets::combo_box::ComboBox;
use qt_widgets::dialog::Dialog;
use qt_widgets::double_spin_box::DoubleSpinBox;
use qt_widgets::file_dialog::{FileDialog, FileMode};
use qt_widgets::grid_layout::GridLayout;
use qt_widgets::group_box::GroupBox;
use qt_widgets::label::Label;
use qt_widgets::layout::Layout;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::main_window::MainWindow;
use qt_widgets::message_box::{MessageBox, Icon};
use qt_widgets::push_button::PushButton;
use qt_widgets::radio_button::RadioButton;
use qt_widgets::tree_view::TreeView;
use qt_widgets::widget::Widget;

use qt_gui::brush::Brush;
use qt_gui::icon;
use qt_gui::key_sequence::KeySequence;
use qt_gui::list::ListStandardItemMutPtr;
use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::connection::Signal;
use qt_core::flags::Flags;
use qt_core::item_selection::ItemSelection;
use qt_core::model_index::ModelIndex;
use qt_core::object::Object;
use qt_core::qt::{GlobalColor, ShortcutContext};
use qt_core::slots::{SlotBool, SlotNoArgs, SlotModelIndexRef};
use qt_core::variant::Variant;
use cpp_utils::StaticCast;

use chrono::NaiveDateTime;
use std::collections::BTreeMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver};
use std::cmp::Ordering;
use std::path::PathBuf;
use std::{fmt, fmt::Display, fmt::Debug};
use std::f32;

use RPFM_PATH;
use SHORTCUTS;
use SETTINGS;
use TREEVIEW_ICONS;
use QString;
use AppUI;
use Commands;
use Data;
use common::*;
use common::communications::*;
use error::{Error, ErrorKind, Result};
use packedfile::*;
use packedfile::db::*;
use packedfile::db::schemas::*;
use packedfile::loc::*;

pub mod dependency_manager;
pub mod packedfile_db;
pub mod packedfile_loc;
pub mod packedfile_text;
pub mod packedfile_image;
pub mod packedfile_rigidmodel;
pub mod settings;
pub mod shortcuts;
pub mod table_state;
pub mod updater;
pub mod qt_custom_stuff;

//----------------------------------------------------------------------------//
//             UI Structs (to hold slots, actions and what not)
//----------------------------------------------------------------------------//

/// This struct holds all the "MyMod" actions from the Menu Bar.
#[derive(Copy, Clone)]
pub struct MyModStuff {
    pub new_mymod: *mut Action,
    pub delete_selected_mymod: *mut Action,
    pub install_mymod: *mut Action,
    pub uninstall_mymod: *mut Action,
}

/// This struct holds all the Slots related to the "MyMod" Menu, as otherwise they'll die before we
/// press their buttons and do nothing.
pub struct MyModSlots {
    pub new_mymod: SlotBool<'static>,
    pub delete_selected_mymod: SlotBool<'static>,
    pub install_mymod: SlotBool<'static>,
    pub uninstall_mymod: SlotBool<'static>,
    pub open_mymod: Vec<SlotBool<'static>>,
}

/// This struct holds all the Slots related to the "Add from PackFile" View, as otherwise they'll
/// die before we press their buttons and do nothing.
pub struct AddFromPackFileSlots {
    pub copy: SlotModelIndexRef<'static>,
    pub exit: SlotNoArgs<'static>,
    pub slot_tree_view_expand_all: SlotNoArgs<'static>,
    pub slot_tree_view_collapse_all: SlotNoArgs<'static>,
}

//----------------------------------------------------------------------------//
//           UI Struct Implementations (impl of the structs above)
//----------------------------------------------------------------------------//

/// Implementation of "AddFromPackFileSlots".
impl AddFromPackFileSlots {

    /// This function creates a new "AddFromPackFileSlots" struct and returns it. This is just for
    /// initialization when starting the program.
    pub fn new() -> Self {

        // Create some dummy slots and return them.
        Self {
            copy: SlotModelIndexRef::new(|_| {}),
            exit: SlotNoArgs::new(|| {}),
            slot_tree_view_expand_all: SlotNoArgs::new(|| {}),
            slot_tree_view_collapse_all: SlotNoArgs::new(|| {}),
        }
    }

    /// This function creates a new "Add From PackFile" struct and returns it.
    pub fn new_with_grid(
        sender_qt: Sender<Commands>,
        sender_qt_data: &Sender<Data>,
        receiver_qt: &Rc<RefCell<Receiver<Data>>>,
        app_ui: AppUI,
        is_folder_tree_view_locked: &Rc<RefCell<bool>>,
        is_modified: &Rc<RefCell<bool>>,
        packedfiles_open_in_packedfile_view: &Rc<RefCell<BTreeMap<i32, Rc<RefCell<Vec<String>>>>>>,
        global_search_explicit_paths: &Rc<RefCell<Vec<Vec<String>>>>,
        update_global_search_stuff: *mut Action,
    ) -> Self {

        // Create the widget that'll act as a container for the view.
        let widget = Widget::new().into_raw();
        let widget_layout = GridLayout::new().into_raw();
        unsafe { widget.as_mut().unwrap().set_layout(widget_layout as *mut Layout); }
        unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(0, widget); }
        
        // Create the stuff.
        let tree_view = TreeView::new().into_raw();
        let tree_model = StandardItemModel::new(()).into_raw();
        let exit_button = PushButton::new(&QString::from_std_str("Exit 'Add from Packfile' Mode")).into_raw();

        // Configure it.
        unsafe { tree_view.as_mut().unwrap().set_model(tree_model as *mut AbstractItemModel); }
        unsafe { tree_view.as_mut().unwrap().set_header_hidden(true); }
        unsafe { tree_view.as_mut().unwrap().set_expands_on_double_click(false); }
        unsafe { tree_view.as_mut().unwrap().set_animated(true); }

        // Add all the stuff to the Grid.
        unsafe { widget_layout.as_mut().unwrap().add_widget((exit_button as *mut Widget, 0, 0, 1, 1)); }
        unsafe { widget_layout.as_mut().unwrap().add_widget((tree_view as *mut Widget, 1, 0, 1, 1)); }

        // Create the slots for the stuff we need.
        let slots = Self {

            // This slot is used to copy something from one PackFile to the other when pressing the "<=" button.
            copy: SlotModelIndexRef::new(clone!(
                global_search_explicit_paths,
                is_modified,
                sender_qt,
                sender_qt_data,
                receiver_qt => move |_| {

                    // Get the file to get from the Right TreeView.
                    let selection_file_to_move;
                    unsafe { selection_file_to_move = tree_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }

                    // Get his path.
                    let item_path = get_path_from_item_selection(tree_model, &selection_file_to_move, true);

                    // Ask the Background Thread to move the files, and send him the path.
                    sender_qt.send(Commands::AddPackedFileFromPackFile).unwrap();
                    sender_qt_data.send(Data::VecString(item_path)).unwrap();

                    // Disable the Main Window (so we can't do other stuff).
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                    // Check what response we got.
                    match check_message_validity_tryrecv(&receiver_qt) {
                    
                        // If it's success....
                        Data::VecVecString(mut paths) => {

                            // Update the TreeView.
                            update_treeview(
                                &sender_qt,
                                &sender_qt_data,
                                receiver_qt.clone(),
                                app_ui.window,
                                app_ui.folder_tree_view,
                                app_ui.folder_tree_model,
                                TreeViewOperation::Add(paths.to_vec()),
                            );

                            // Set the mod as "Modified". This is an exception for the path, as it'll be painted later on.
                            *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                            // Update the global search stuff, if needed.
                            global_search_explicit_paths.borrow_mut().append(&mut paths);
                            unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }
                        }

                        // If we got an error...
                        Data::Error(error) => show_dialog(app_ui.window, true, error),

                        // In ANY other situation, it's a message problem.
                        _ => panic!(THREADS_MESSAGE_ERROR),
                    }

                    // Re-enable the Main Window.
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

                    // Set the focus again on the extra Treeview, so we don't need to refocus manually.
                    unsafe { tree_view.as_mut().unwrap().set_focus(()); }
                }
            )),

            // This slot is used to exit the "Add from PackFile" view, returning to the normal state of the program.
            exit: SlotNoArgs::new(clone!(
                sender_qt,
                packedfiles_open_in_packedfile_view,
                is_folder_tree_view_locked => move || {

                    // Reset the Secondary PackFile.
                    sender_qt.send(Commands::ResetPackFileExtra).unwrap();

                    // Destroy the "Add from PackFile" stuff.
                    purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);

                    // Show the "Tips".
                    display_help_tips(&app_ui);

                    // Unlock the TreeView so it can load PackedFiles again.
                    *is_folder_tree_view_locked.borrow_mut() = false;
                }
            )),

            // Actions without buttons for the TreeView.
            slot_tree_view_expand_all: SlotNoArgs::new(move || { unsafe { tree_view.as_mut().unwrap().expand_all(); }}),
            slot_tree_view_collapse_all: SlotNoArgs::new(move || { unsafe { tree_view.as_mut().unwrap().collapse_all(); }}),
        };

        let tree_view_expand_all = Action::new(&QString::from_std_str("&Expand All")).into_raw();
        let tree_view_collapse_all = Action::new(&QString::from_std_str("&Collapse All")).into_raw();
        
        // Actions for the slots...
        unsafe { tree_view.as_ref().unwrap().signals().double_clicked().connect(&slots.copy); }
        unsafe { exit_button.as_ref().unwrap().signals().released().connect(&slots.exit); }
        unsafe { tree_view_expand_all.as_ref().unwrap().signals().triggered().connect(&slots.slot_tree_view_expand_all); }
        unsafe { tree_view_collapse_all.as_ref().unwrap().signals().triggered().connect(&slots.slot_tree_view_collapse_all); }

        unsafe { tree_view_expand_all.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().tree_view.get("expand_all").unwrap()))); }
        unsafe { tree_view_collapse_all.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().tree_view.get("collapse_all").unwrap()))); }

        unsafe { tree_view_expand_all.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { tree_view_collapse_all.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }

        unsafe { tree_view.as_mut().unwrap().add_action(tree_view_expand_all); }
        unsafe { tree_view.as_mut().unwrap().add_action(tree_view_collapse_all); }

        // Update the new TreeView.
        update_treeview(
            &sender_qt,
            &sender_qt_data,
            receiver_qt.clone(),
            app_ui.window,
            tree_view,
            tree_model,
            TreeViewOperation::Build(true),
        );

        // Return the slots, to be kept alive.
        slots
    }
}

//----------------------------------------------------------------------------//
//             UI Creation functions (to build the UI on start)
//----------------------------------------------------------------------------//

/// This function creates the entire "Rename Current" dialog. It returns the new name of the Item, or
/// None if the dialog is canceled or closed.
pub fn create_rename_dialog(app_ui: &AppUI, name: &str) -> Option<String> {

    //-------------------------------------------------------------------------------------------//
    // Creating the Rename Dialog...
    //-------------------------------------------------------------------------------------------//

    // Create the "Rename" Dialog.
    let mut dialog;
    unsafe { dialog = Dialog::new_unsafe(app_ui.window as *mut Widget); }

    // Change his title.
    dialog.set_window_title(&QString::from_std_str("Rename"));
    dialog.resize((400, 50));

    // Set it Modal, so you can't touch the Main Window with this dialog open.
    dialog.set_modal(true);

    // Create the main Grid.
    let main_grid = GridLayout::new().into_raw();

    // Create the "New Name" LineEdit.
    let mut new_name_line_edit = LineEdit::new(());

    // Set the current name as default.
    new_name_line_edit.set_text(&QString::from_std_str(name));

    // Create the "Rename" button.
    let rename_button = PushButton::new(&QString::from_std_str("Rename")).into_raw();

    // Add all the widgets to the main grid.
    unsafe { main_grid.as_mut().unwrap().add_widget((new_name_line_edit.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((rename_button as *mut Widget, 0, 1, 1, 1)); }

    // And the Main Grid to the Dialog...
    unsafe { dialog.set_layout(main_grid as *mut Layout); }

    //-------------------------------------------------------------------------------------------//
    // Actions for the Rename Dialog...
    //-------------------------------------------------------------------------------------------//

    // What happens when we hit the "Rename" button.
    unsafe { rename_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

    // Show the Dialog and, if we hit the "Rename" button...
    if dialog.exec() == 1 {

        // Get the text from the LineEdit.
        let new_name = new_name_line_edit.text().to_std_string();

        // Return the new name.
        Some(new_name)
    }

    // Otherwise, return None.
    else { None }
}

/// This function creates the entire "Apply Prefix to Selected/All" dialog. It returns the prefix for the items, or
/// None if the dialog is canceled or closed.
pub fn create_apply_prefix_to_packed_files_dialog(app_ui: &AppUI) -> Option<String> {

    // Create the Dialog and configure it.
    let mut dialog;
    unsafe { dialog = Dialog::new_unsafe(app_ui.window as *mut Widget); }
    dialog.set_window_title(&QString::from_std_str("Apply Prefix to Selected/All"));
    dialog.set_modal(true);

    // Create the main Grid.
    let main_grid = GridLayout::new().into_raw();
    let mut add_prefix_line_edit = LineEdit::new(());
    add_prefix_line_edit.set_placeholder_text(&QString::from_std_str("Write a prefix here, like 'mua_'."));

    // Create the "Apply" button.
    let rename_button = PushButton::new(&QString::from_std_str("Apply Prefix")).into_raw();
    unsafe { main_grid.as_mut().unwrap().add_widget((add_prefix_line_edit.static_cast_mut() as *mut Widget, 0, 1, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((rename_button as *mut Widget, 0, 2, 1, 1)); }

    // And the Main Grid to the Dialog...
    unsafe { dialog.set_layout(main_grid as *mut Layout); }

    // What happens when we hit the "Rename" button.
    unsafe { rename_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

    // Show the Dialog and, if we hit the button, return the prefix.
    if dialog.exec() == 1 { Some(add_prefix_line_edit.text().to_std_string()) }

    // Otherwise, return None.
    else { None }
}
/// This function creates the entire "New Folder" dialog. It returns the new name of the Folder, or
/// None if the dialog is canceled or closed.
pub fn create_new_folder_dialog(app_ui: &AppUI) -> Option<String> {

    //-------------------------------------------------------------------------------------------//
    // Creating the New Folder Dialog...
    //-------------------------------------------------------------------------------------------//

    // Create the "New Folder" Dialog.
    let mut dialog;
    unsafe { dialog = Dialog::new_unsafe(app_ui.window as *mut Widget); }

    // Change his title.
    dialog.set_window_title(&QString::from_std_str("New Folder"));

    // Set it Modal, so you can't touch the Main Window with this dialog open.
    dialog.set_modal(true);

    // Create the main Grid.
    let main_grid = GridLayout::new().into_raw();

    // Create the "New Folder" LineEdit.
    let mut new_folder_line_edit = LineEdit::new(());

    // Set the current name as default.
    new_folder_line_edit.set_text(&QString::from_std_str("new_folder"));

    // Create the "New Folder" button.
    let new_folder_button = PushButton::new(&QString::from_std_str("New Folder")).into_raw();

    // Add all the widgets to the main grid.
    unsafe { main_grid.as_mut().unwrap().add_widget((new_folder_line_edit.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((new_folder_button as *mut Widget, 0, 1, 1, 1)); }

    // And the Main Grid to the Dialog...
    unsafe { dialog.set_layout(main_grid as *mut Layout); }

    //-------------------------------------------------------------------------------------------//
    // Actions for the New Folder Dialog...
    //-------------------------------------------------------------------------------------------//

    // What happens when we hit the "New Folder" button.
    unsafe { new_folder_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

    // Show the Dialog and, if we hit the "New Folder" button...
    if dialog.exec() == 1 {

        // Get the text from the LineEdit.
        let new_name = new_folder_line_edit.text().to_std_string();

        // Return the new name.
        Some(new_name)
    }

    // Otherwise, return None.
    else { None }
}

/// This function creates all the "New PackedFile" dialogs. It returns the type/name of the new file,
/// or None if the dialog is canceled or closed.
pub fn create_new_packed_file_dialog(
    app_ui: &AppUI,
    sender: &Sender<Commands>,
    sender_data: &Sender<Data>,
    receiver: &Rc<RefCell<Receiver<Data>>>,
    packed_file_type: PackedFileType
) -> Option<Result<PackedFileType>> {

    //-------------------------------------------------------------------------------------------//
    // Creating the New PackedFile Dialog...
    //-------------------------------------------------------------------------------------------//

    // Create and configure the "New PackedFile" Dialog.
    let mut dialog = unsafe { Dialog::new_unsafe(app_ui.window as *mut Widget) };
    match packed_file_type {
        PackedFileType::Loc(_) => dialog.set_window_title(&QString::from_std_str("New Loc PackedFile")),
        PackedFileType::DB(_,_,_) => dialog.set_window_title(&QString::from_std_str("New DB Table")),
        PackedFileType::Text(_) => dialog.set_window_title(&QString::from_std_str("New Text PackedFile")),
    }
    dialog.set_modal(true);

    // Create the main Grid and his widgets.
    let main_grid = GridLayout::new().into_raw();
    let mut new_packed_file_name_edit = LineEdit::new(());
    let create_button = PushButton::new(&QString::from_std_str("Create")).into_raw();
    let mut table_dropdown = ComboBox::new();
    let mut table_model = StandardItemModel::new(());
    unsafe { dialog.set_layout(main_grid as *mut Layout); }

    new_packed_file_name_edit.set_text(&QString::from_std_str("new_file"));
    unsafe { table_dropdown.set_model(table_model.static_cast_mut()); }

    // Add all the widgets to the main grid.
    unsafe { main_grid.as_mut().unwrap().add_widget((new_packed_file_name_edit.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((create_button as *mut Widget, 0, 1, 1, 1)); }

    // If it's a DB Table...
    if let PackedFileType::DB(_,_,_) = packed_file_type {

        // Get a list of all the tables currently in use by the selected game.
        sender.send(Commands::GetTableListFromDependencyPackFile).unwrap();
        let tables = if let Data::VecString(data) = check_message_validity_recv2(&receiver) { data } else { panic!(THREADS_MESSAGE_ERROR); };

        // Get the current schema.
        sender.send(Commands::GetSchema).unwrap();
        let schema = if let Data::OptionSchema(data) = check_message_validity_recv2(&receiver) { data } else { panic!(THREADS_MESSAGE_ERROR); };

        // Check if we actually have an schema.
        match schema {

            // If we have an schema...
            Some(schema) => {

                // Add every table to the dropdown if exists in the dependency database.
                schema.tables_definitions.iter().filter(|x| tables.contains(&x.name)).for_each(|x| table_dropdown.add_item(&QString::from_std_str(&x.name)));
                unsafe { main_grid.as_mut().unwrap().add_widget((table_dropdown.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
            }

            // If we don't have an schema, return Some(Error).
            None => return Some(Err(Error::from(ErrorKind::SchemaNotFound))),
        }
    }

    //-------------------------------------------------------------------------------------------//
    // Actions for the New PackedFile Dialog...
    //-------------------------------------------------------------------------------------------//

    // What happens when we hit the "Create" button.
    unsafe { create_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

    // Show the Dialog and, if we hit the "Create" button...
    if dialog.exec() == 1 {

        // Get the text from the LineEdit.
        let packed_file_name = new_packed_file_name_edit.text().to_std_string();

        // Depending on the PackedFile's Type, return the new name.
        match packed_file_type {
            PackedFileType::Loc(_) => Some(Ok(PackedFileType::Loc(packed_file_name))),
            PackedFileType::DB(_,_,_) => {

                // Get the table and his version.
                let table = table_dropdown.current_text().to_std_string();

                // Get the data of the table used in the dependency database.
                sender.send(Commands::GetTableVersionFromDependencyPackFile).unwrap();
                sender_data.send(Data::String(table.to_owned())).unwrap();
                let version = match check_message_validity_recv2(&receiver) { 
                    Data::U32(data) => data,
                    Data::Error(error) => return Some(Err(error)),
                    _ => panic!(THREADS_MESSAGE_ERROR), 
                };
                Some(Ok(PackedFileType::DB(packed_file_name, table, version)))
            },
            PackedFileType::Text(_) => Some(Ok(PackedFileType::Text(packed_file_name))),
        }
    }

    // Otherwise, return None.
    else { None }
}

/// This function creates the "Mass-Import TSV" dialog. Nothing too massive. It returns the name of
/// the new imported PackedFiles & their Paths, or None in case of closing the dialog.
pub fn create_mass_import_tsv_dialog(app_ui: &AppUI) -> Option<(String, Vec<PathBuf>)> {

    //-------------------------------------------------------------------------------------------//
    // Creating the Mass-Import TSV Dialog...
    //-------------------------------------------------------------------------------------------//

    // Create the "Mass-Import TSV" Dialog and configure it.
    let dialog = unsafe { Dialog::new_unsafe(app_ui.window as *mut Widget).into_raw() };
    unsafe { dialog.as_mut().unwrap().set_window_title(&QString::from_std_str("Mass-Import TSV Files")); }
    unsafe { dialog.as_mut().unwrap().set_modal(true); }
    unsafe { dialog.as_mut().unwrap().resize((400, 100)); }

    // Create the main Grid and his stuff.
    let main_grid = GridLayout::new().into_raw();
    let files_to_import_label = Label::new(&QString::from_std_str("Files to import: 0.")).into_raw();
    let select_files_button = PushButton::new(&QString::from_std_str("...")).into_raw();
    let mut imported_files_name_line_edit = LineEdit::new(());
    let import_button = PushButton::new(&QString::from_std_str("Import")).into_raw();

    // Set a dummy name as default.
    imported_files_name_line_edit.set_text(&QString::from_std_str("new_imported_file"));

    // Add all the widgets to the main grid, and the main grid to the dialog.
    unsafe { main_grid.as_mut().unwrap().add_widget((files_to_import_label as *mut Widget, 0, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((select_files_button as *mut Widget, 0, 1, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((imported_files_name_line_edit.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((import_button as *mut Widget, 1, 1, 1, 1)); }
    unsafe { dialog.as_mut().unwrap().set_layout(main_grid as *mut Layout); }

    //-------------------------------------------------------------------------------------------//
    // Actions for the Mass-Import TSV Dialog...
    //-------------------------------------------------------------------------------------------//

    // Create the list of Paths to import.
    let files_to_import = Rc::new(RefCell::new(vec![]));

    // What happens when we hit the "..." button.
    let slot_select_files = SlotNoArgs::new(clone!(
        files_to_import => move || {

            // Create the FileDialog to get the TSV files, and add them to the list if we accept.
            let mut file_dialog = unsafe { FileDialog::new_unsafe((
                dialog as *mut Widget,
                &QString::from_std_str("Select TSV Files to Import..."),
            )) };

            file_dialog.set_name_filter(&QString::from_std_str("TSV Files (*.tsv)"));
            file_dialog.set_file_mode(FileMode::ExistingFiles);

            if file_dialog.exec() == 1 {
                let selected_files = file_dialog.selected_files();
                files_to_import.borrow_mut().clear();
                for index in 0..selected_files.count(()) {
                    files_to_import.borrow_mut().push(PathBuf::from(file_dialog.selected_files().at(index).to_std_string()));
                }

                unsafe { files_to_import_label.as_mut().unwrap().set_text(&QString::from_std_str(&format!("Files to import: {}.", selected_files.count(())))); }
            }
        }
    ));

    unsafe { select_files_button.as_mut().unwrap().signals().released().connect(&slot_select_files); }
    unsafe { import_button.as_mut().unwrap().signals().released().connect(&dialog.as_mut().unwrap().slots().accept()); }

    // If we hit the "Create" button, take the name you wrote and the list of files, and return them.
    if unsafe { dialog.as_mut().unwrap().exec() } == 1 {
        let packed_file_name = imported_files_name_line_edit.text().to_std_string();
        Some((packed_file_name, files_to_import.borrow().to_vec()))
    }

    // In any other case, we return None.
    else { None }
}

/*
/// This function serves as a common function to all the "Create Prefab" buttons from "Special Stuff".
fn create_prefab(
    application: &Application,
    app_ui: &AppUI,
    game_selected: &Rc<RefCell<GameSelected>>,
    pack_file_decoded: &Rc<RefCell<PackFile>>,
) {
    // Create the list of PackedFiles to "move".
    let mut prefab_catchments: Vec<usize> = vec![];

    // For each PackedFile...
    for (index, packed_file) in pack_file_decoded.borrow().data.packed_files.iter().enumerate() {

        // If it's in the exported map's folder...
        if packed_file.path.starts_with(&["terrain".to_owned(), "tiles".to_owned(), "battle".to_owned(), "_assembly_kit".to_owned()]) {

            // Get his name.
            let packed_file_name = packed_file.path.last().unwrap();

            // If it's one of the exported layers...
            if packed_file_name.starts_with("catchment") && packed_file_name.ends_with(".bin") {

                // Add it to the list.
                prefab_catchments.push(index);
            }
        }
    }

    // If we found at least one catchment PackedFile...
    if !prefab_catchments.is_empty() {

        // Disable the main window, so the user can't do anything until all the prefabs are processed.
        app_ui.window.set_sensitive(false);

        // We create a "New Prefabs" window.
        NewPrefabWindow::create_new_prefab_window(
            &app_ui,
            application,
            game_selected,
            pack_file_decoded,
            &prefab_catchments
        );
    }

    // If there are not suitable PackedFiles...
    else { show_dialog(app_ui.window, false, "There are no catchment PackedFiles in this PackFile."); }
}*/

/// This function creates the entire "Global Search" dialog. It returns the search info (pattern, case_sensitive).
pub fn create_global_search_dialog(app_ui: &AppUI) -> Option<String> {

    let mut dialog;
    unsafe { dialog = Dialog::new_unsafe(app_ui.window as *mut Widget); }
    dialog.set_window_title(&QString::from_std_str("Global Search"));
    dialog.set_modal(true);

    // Create the main Grid.
    let main_grid = GridLayout::new().into_raw();
    let mut pattern = LineEdit::new(());
    pattern.set_placeholder_text(&QString::from_std_str("Write here what you want to search."));

    let search_button = PushButton::new(&QString::from_std_str("Search")).into_raw();
    unsafe { main_grid.as_mut().unwrap().add_widget((pattern.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((search_button as *mut Widget, 0, 1, 1, 1)); }
    unsafe { dialog.set_layout(main_grid as *mut Layout); }

    // What happens when we hit the "Search" button.
    unsafe { search_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

    // Execute the dialog.
    if dialog.exec() == 1 { 
        let text = pattern.text().to_std_string();
        if !text.is_empty() { Some(text) }
        else { None }
    }
    
    // Otherwise, return None.
    else { None }
}

/// This function creates the entire "Apply Maths" dialog for tables. It returns the operation to apply and the value.
pub fn create_apply_maths_dialog(app_ui: &AppUI) -> Option<(String, f64)> {

    // Create and configure the "Apply Maths" Dialog.
    let mut dialog = unsafe { Dialog::new_unsafe(app_ui.window as *mut Widget) };
    dialog.set_window_title(&QString::from_std_str("Apply Maths"));
    dialog.set_modal(true);
    let main_grid = GridLayout::new().into_raw();

    // Create the button group with the different operations, and set by default the "+" selected.
    let operations_frame = GroupBox::new(&QString::from_std_str("Operations")).into_raw();
    let operations_grid = GridLayout::new().into_raw();
    unsafe { operations_frame.as_mut().unwrap().set_layout(operations_grid as *mut Layout); }

    let mut button_group = ButtonGroup::new();
    let mut operation_plus = RadioButton::new(&QString::from_std_str("+"));
    let mut operation_minus = RadioButton::new(&QString::from_std_str("-"));
    let mut operation_mult = RadioButton::new(&QString::from_std_str("*"));
    let mut operation_div = RadioButton::new(&QString::from_std_str("/"));
    unsafe { button_group.add_button(operation_plus.static_cast_mut() as *mut AbstractButton); }
    unsafe { button_group.add_button(operation_minus.static_cast_mut() as *mut AbstractButton); }
    unsafe { button_group.add_button(operation_mult.static_cast_mut() as *mut AbstractButton); }
    unsafe { button_group.add_button(operation_div.static_cast_mut() as *mut AbstractButton); }
    operation_plus.click();

    // Create a little frame with some instructions.
    let instructions_frame = GroupBox::new(&QString::from_std_str("Instructions")).into_raw();
    let instructions_grid = GridLayout::new().into_raw();
    unsafe { instructions_frame.as_mut().unwrap().set_layout(instructions_grid as *mut Layout); }
    let mut instructions_label = Label::new(&QString::from_std_str(
    "It's easy:
     - Choose the operation on the left.
     - Write the operand on the SpinBox.
     - Click the button on the right.
     - ???
     - Profit!
    "    
    ));
    unsafe { instructions_grid.as_mut().unwrap().add_widget((instructions_label.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }

    // We use a double SpinBox for the value, so we can do any operation with F32 floats.
    let mut value_spin_box = DoubleSpinBox::new();
    value_spin_box.set_decimals(3);
    value_spin_box.set_range(f32::MIN as f64, f32::MAX as f64);
    let apply_button = PushButton::new(&QString::from_std_str("Apply")).into_raw();

    unsafe { operations_grid.as_mut().unwrap().add_widget((operation_plus.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
    unsafe { operations_grid.as_mut().unwrap().add_widget((operation_minus.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
    unsafe { operations_grid.as_mut().unwrap().add_widget((operation_mult.static_cast_mut() as *mut Widget, 2, 0, 1, 1)); }
    unsafe { operations_grid.as_mut().unwrap().add_widget((operation_div.static_cast_mut() as *mut Widget, 3, 0, 1, 1)); }

    unsafe { main_grid.as_mut().unwrap().add_widget((operations_frame as *mut Widget, 0, 0, 4, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((instructions_frame as *mut Widget, 1, 1, 3, 2)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((value_spin_box.static_cast_mut() as *mut Widget, 0, 1, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((apply_button as *mut Widget, 0, 2, 1, 1)); }
    unsafe { dialog.set_layout(main_grid as *mut Layout); }

    unsafe { apply_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

    if dialog.exec() == 1 {
        let operation = unsafe { button_group.checked_button().as_ref().unwrap().text().to_std_string() };
        let value = value_spin_box.value();
        Some((operation, value))
    } else { None }
}

/// This function creates the entire "Apply Prefix" dialog for tables. It returns the prefix to apply, or None.
pub fn create_apply_prefix_dialog(app_ui: &AppUI) -> Option<String> {

    // Create and configure the "Apply Maths" Dialog.
    let mut dialog = unsafe { Dialog::new_unsafe(app_ui.window as *mut Widget) };
    dialog.set_window_title(&QString::from_std_str("Apply Prefix"));
    dialog.set_modal(true);
    dialog.resize((400, 50));
    let main_grid = GridLayout::new().into_raw();

    let mut prefix_line_edit = LineEdit::new(());
    prefix_line_edit.set_placeholder_text(&QString::from_std_str("Write here the prefix you want."));
    let apply_button = PushButton::new(&QString::from_std_str("Apply")).into_raw();

    unsafe { main_grid.as_mut().unwrap().add_widget((prefix_line_edit.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((apply_button as *mut Widget, 0, 1, 1, 1)); }
    unsafe { dialog.set_layout(main_grid as *mut Layout); }

    unsafe { apply_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

    if dialog.exec() == 1 { 
        let prefix = prefix_line_edit.text().to_std_string();
        if prefix.is_empty() { None } else { Some(prefix_line_edit.text().to_std_string()) } 
    } else { None }
}

//----------------------------------------------------------------------------//
//                    Trait Implementations for Qt Stuff
//----------------------------------------------------------------------------//

/// Rust doesn't allow implementing traits for types you don't own, so we have to wrap ModelIndex for ordering it.
/// Don't like it a bit.
pub struct ModelIndexWrapped {
    pub model_index: ModelIndex
}

impl ModelIndexWrapped {
    pub fn new(model_index: ModelIndex) -> Self {
        ModelIndexWrapped {
            model_index
        }
    }

    pub fn get(&self) -> &ModelIndex {
        &self.model_index
    }
}

impl Ord for ModelIndexWrapped {
    fn cmp(&self, other: &ModelIndexWrapped) -> Ordering {
        let order = self.model_index.row().cmp(&other.model_index.row());
        if order == Ordering::Equal { self.model_index.column().cmp(&other.model_index.column()) }
        else { order }
    }
}

impl PartialOrd for ModelIndexWrapped {
    fn partial_cmp(&self, other: &ModelIndexWrapped) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for ModelIndexWrapped {}
impl PartialEq for ModelIndexWrapped {
    fn eq(&self, other: &ModelIndexWrapped) -> bool {
        self.model_index.row() == other.model_index.row() && self.model_index.column() == other.model_index.column()
    }
}

//----------------------------------------------------------------------------//
//                  Undo/Redo stuff for Tables and Locs
//----------------------------------------------------------------------------//

/// This function is used to update the background or undo table when a change is made in the main table.
fn update_undo_model(model: *mut StandardItemModel, undo_model: *mut StandardItemModel) {
    unsafe {
        undo_model.as_mut().unwrap().clear();
        for row in 0..model.as_mut().unwrap().row_count(()) {
            for column in 0..model.as_mut().unwrap().column_count(()) {
                let item = &*model.as_mut().unwrap().item((row, column));
                undo_model.as_mut().unwrap().set_item((row, column, item.clone()));
            }    
        }
    }
}

//----------------------------------------------------------------------------//
//                    Enums & Structs needed for the UI
//----------------------------------------------------------------------------//

/// Enum `TreeViewOperation`: This enum has the different possible operations we want to do over a TreeView. The options are:
/// - `Build`: Build the entire TreeView from nothing. Requires a bool, depending if the PackFile is editable or not.
/// - `Add`: Add a File/Folder to the TreeView. Requires the path in the TreeView, without the mod's name.
/// - `DeleteSelected`: Removes whatever is selected from the TreeView. It requires the TreePathType of whatever you want to delete.
/// - `DeleteUnselected`: Remove the File/Folder corresponding to the TreePathType we provide from the TreeView. It requires the TreePathType of whatever you want to delete.
/// - `Rename`: Change the name of a File/Folder from the TreeView. Requires the TreePathType of whatever you want to rename and the new name.
/// - `PrefixFiles`: Apply a prefix to every file under certain folder. Requires the old paths and the prefix to apply.
#[derive(Clone, Debug)]
pub enum TreeViewOperation {
    Build(bool),
    Add(Vec<Vec<String>>),
    DeleteSelected(TreePathType),
    DeleteUnselected(TreePathType),
    Rename(TreePathType, String),
    PrefixFiles(Vec<Vec<String>>, String),
}

/// Enum `ItemVisualStatus`: This enum represents the status of modification of an item in a TreeView.
#[derive(PartialEq)]
pub enum ItemVisualStatus {
    Added,
    Modified,
    AddedModified,
    Untouched,
}

/// Enum to know what operation was done while editing tables, so we can revert them with undo.
/// - Editing: Intended for any kind of item editing. Holds a Vec<((row, column), *mut item)>, so we can do this in batches.
/// - AddRows: Intended for when adding/inserting rows. It holds a list of positions where the rows where inserted.
/// - RemoveRows: Intended for when removing rows. It holds a list of positions where the rows where deleted and the deleted rows.
/// - SmartDelete: Intended for when we are using the smart delete feature. This is a combination of list of edits and list of removed rows.
/// - RevertSmartDelete: Selfexplanatory. This is a combination of list of edits and list of adding rows.
/// - ImportTSVDB: It holds a copy of the entire DB, before importing.
/// - ImportTSVLOC: It holds a copy of the entire Loc, before importing.
pub enum TableOperations {
    Editing(Vec<((i32, i32), *mut StandardItem)>),
    AddRows(Vec<i32>),
    RemoveRows((Vec<i32>, Vec<ListStandardItemMutPtr>)),
    SmartDelete((Vec<((i32, i32), *mut StandardItem)>, Vec<(i32, ListStandardItemMutPtr)>)),
    RevertSmartDelete((Vec<((i32, i32), *mut StandardItem)>, Vec<i32>)),
    ImportTSVDB(DB),
    ImportTSVLOC(Loc),
}

/// Enum `IconType`: This enum holds all the possible Icon Types we can have in the TreeView,
/// depending on the type of the PackedFiles.
enum IconType {

    // For normal PackFiles. True for editable, false for read-only.
    PackFile(bool),

    // For folders.
    Folder,

    // For files. Includes the path without the Packfile.
    File(Vec<String>),
}

/// Struct `Icons`. This struct is used to hold all the Qt Icons used by the TreeView. This is generated
/// everytime we call "update_treeview", but ideally we should move it to on start.
pub struct Icons {
    pub packfile_editable: icon::Icon,
    pub packfile_locked: icon::Icon,
    pub folder: icon::Icon,

    // For generic files.
    pub file: icon::Icon,

    // For tables and loc files.
    pub table: icon::Icon,

    // For images.
    pub image_generic: icon::Icon,
    pub image_png: icon::Icon,
    pub image_jpg: icon::Icon,

    // For text files.
    pub text_generic: icon::Icon,
    pub text_csv: icon::Icon,
    pub text_html: icon::Icon,
    pub text_txt: icon::Icon,
    pub text_xml: icon::Icon,

    // For rigidmodels.
    pub rigid_model: icon::Icon,
}

/// Debug implementation of TableOperations, so we can at least guess what is in the history.
impl Debug for TableOperations {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TableOperations::Editing(data) => write!(f, "Cell/s edited, starting in row {}, column {}.", (data[0].0).0, (data[0].0).1),
            TableOperations::AddRows(data) => write!(f, "Row/s added in position/s {}.", data.iter().map(|x| format!("{}, ", x)).collect::<String>()),
            TableOperations::RemoveRows(data) => write!(f, "Row/s removed in position/s {}.", data.0.iter().map(|x| format!("{}, ", x)).collect::<String>()),
            TableOperations::SmartDelete(_) => write!(f, "Smart deletion."),
            TableOperations::RevertSmartDelete(_) => write!(f, "Reverted Smart deletion."),
            TableOperations::ImportTSVDB(_) | TableOperations::ImportTSVLOC(_) => write!(f, "Imported TSV file."),
        }
    }
}

/// Implementation of "Icons".
impl Icons {

    /// This function creates a list of Icons from certain paths in disk.
    pub fn new() -> Self {

        // Get the Path as a String, so Qt can understand it.
        let rpfm_path_string = RPFM_PATH.to_string_lossy().as_ref().to_string();

        // Prepare the path for the icons of the TreeView.
        let mut icon_packfile_editable_path = rpfm_path_string.to_owned();
        let mut icon_packfile_locked_path = rpfm_path_string.to_owned();
        let mut icon_folder_path = rpfm_path_string.to_owned();
        let mut icon_file_path = rpfm_path_string.to_owned();

        let mut icon_table_path = rpfm_path_string.to_owned();

        let mut icon_image_generic_path = rpfm_path_string.to_owned();
        let mut icon_image_png_path = rpfm_path_string.to_owned();
        let mut icon_image_jpg_path = rpfm_path_string.to_owned();

        let mut icon_text_generic_path = rpfm_path_string.to_owned();
        let mut icon_text_csv_path = rpfm_path_string.to_owned();
        let mut icon_text_html_path = rpfm_path_string.to_owned();
        let mut icon_text_txt_path = rpfm_path_string.to_owned();
        let mut icon_text_xml_path = rpfm_path_string.to_owned();

        let mut icon_rigid_model_path = rpfm_path_string.to_owned();

        // Get the Icons for each type of Item.
        icon_packfile_editable_path.push_str("/img/packfile_editable.svg");
        icon_packfile_locked_path.push_str("/img/packfile_locked.svg");
        icon_folder_path.push_str("/img/folder.svg");
        icon_file_path.push_str("/img/file.svg");

        icon_table_path.push_str("/img/database.svg");

        icon_image_generic_path.push_str("/img/generic_image.svg");
        icon_image_png_path.push_str("/img/png.svg");
        icon_image_jpg_path.push_str("/img/jpg.svg");

        icon_text_generic_path.push_str("/img/generic_text.svg");
        icon_text_csv_path.push_str("/img/csv.svg");
        icon_text_html_path.push_str("/img/html.svg");
        icon_text_txt_path.push_str("/img/txt.svg");
        icon_text_xml_path.push_str("/img/xml.svg");

        icon_rigid_model_path.push_str("/img/rigid_model.svg");

        // Get the Icons in Qt Icon format.
        Self {
            packfile_editable: icon::Icon::new(&QString::from_std_str(icon_packfile_editable_path)),
            packfile_locked: icon::Icon::new(&QString::from_std_str(icon_packfile_locked_path)),
            folder: icon::Icon::new(&QString::from_std_str(icon_folder_path)),
            file: icon::Icon::new(&QString::from_std_str(icon_file_path)),

            table: icon::Icon::new(&QString::from_std_str(icon_table_path)),

            image_generic: icon::Icon::new(&QString::from_std_str(icon_image_generic_path)),
            image_png: icon::Icon::new(&QString::from_std_str(icon_image_png_path)),
            image_jpg: icon::Icon::new(&QString::from_std_str(icon_image_jpg_path)),

            text_generic: icon::Icon::new(&QString::from_std_str(icon_text_generic_path)),
            text_csv: icon::Icon::new(&QString::from_std_str(icon_text_csv_path)),
            text_html: icon::Icon::new(&QString::from_std_str(icon_text_html_path)),
            text_txt: icon::Icon::new(&QString::from_std_str(icon_text_txt_path)),
            text_xml: icon::Icon::new(&QString::from_std_str(icon_text_xml_path)),

            rigid_model: icon::Icon::new(&QString::from_std_str(icon_rigid_model_path)),
        }
    }
}

//----------------------------------------------------------------------------//
//              Utility functions (helpers and stuff like that)
//----------------------------------------------------------------------------//

/// This function shows a "Success" or "Error" Dialog with some text. For notification of success and errors.
/// It requires:
/// - window: a pointer to the main window of the program, to set it as a parent..
/// - is_success: true for "Success" Dialog, false for "Error" Dialog.
/// - text: something that implements the trait "Display", so we want to put in the dialog window.
pub fn show_dialog<T: Display>(
    window: *mut MainWindow,
    is_success: bool,
    text: T
) {

    // Depending on the type of the dialog, set everything specific here.
    let title = if is_success { "Success!" } else { "Error!" };
    let icon = if is_success { Icon::Information } else { Icon::Critical };

    // Create the dialog.
    let mut dialog;
    unsafe { dialog = MessageBox::new_unsafe((
        icon,
        &QString::from_std_str(title),
        &QString::from_std_str(&text.to_string()),
        Flags::from_int(1024), // Ok button.
        window as *mut Widget,
    )); }

    // Run the dialog.
    dialog.exec();
}

/// This function sets the currently open PackFile as "modified" or unmodified, both in the PackFile
/// and in the title bar, depending on the value of the "is_modified" boolean. If provided with a path,
/// It also gets that path from the main TreeView and paints it as modified.
/// NOTE: THIS ALWAYS HAS TO GO AFTER A TREEVIEW UPDATE, NEVER BEFORE IT.
pub fn set_modified(
    is_modified: bool,
    app_ui: &AppUI,
    path: Option<Vec<String>>
) -> bool {

    // If the PackFile is modified...
    if is_modified {

        // Get the name of the mod.
        let pack_file_name;
        unsafe { pack_file_name = app_ui.folder_tree_model.as_mut().unwrap().item(0).as_mut().unwrap().text().to_std_string(); }

        // Change the title of the Main Window.
        unsafe { app_ui.window.as_mut().unwrap().set_window_title(&QString::from_std_str(format!("{} - Modified", pack_file_name))); }

        // If we have received a path to mark as "modified"...
        if let Some(path) = path {

            // Get the item of the Path.
            let item = get_item_from_incomplete_path(app_ui.folder_tree_model, &path);

            // Paint the modified item.
            paint_treeview(item, app_ui.folder_tree_model, ItemVisualStatus::Modified);
        }

        // And return true.
        true
    }

    // If it's not modified...
    else {

        // Check if there is a PackFile open.
        let is_pack_file_open;
        unsafe { is_pack_file_open = if app_ui.folder_tree_model.as_mut().unwrap().row_count(()) > 0 { true } else { false }; }

        // If there is no PackFile open...
        if !is_pack_file_open {

            // Change the title of the Main Window.
            unsafe { app_ui.window.as_mut().unwrap().set_window_title(&QString::from_std_str("Rusted PackFile Manager")); }
        }

        // Otherwise...
        else {

            // Get the name of the mod.
            let pack_file_name;
            unsafe { pack_file_name = app_ui.folder_tree_model.as_mut().unwrap().item(0).as_mut().unwrap().text().to_std_string(); }

            // Change the title of the Main Window.
            unsafe { app_ui.window.as_mut().unwrap().set_window_title(&QString::from_std_str(format!("{} - Not Modified", pack_file_name))); }

            // Clean the TreeView from changes.
            unsafe { clean_treeview(app_ui.folder_tree_model.as_mut().unwrap().item(0), app_ui.folder_tree_model); }
        }

        // And return false.
        false
    }
}

/// This function is intended to be triggered when we undo all the way to the begining a table or loc.
/// It "unpaints" it and, checks if the parent should still be painted, and repeats until it finds a parent
/// that should be painted, or reaches the PackFile. If the PackFile should not be painted,
/// then sets the PackFile as "not modified". 
pub fn undo_paint_for_packed_file(
    app_ui: &AppUI,
    model: *mut StandardItemModel,
    path: &Rc<RefCell<Vec<String>>>,
) {

    // Get the item and paint it transparent.
    let item = get_item_from_incomplete_path(app_ui.folder_tree_model, &path.borrow());
    unsafe { item.as_mut().unwrap().set_background(&Brush::new(GlobalColor::Transparent)); }

    // Get the full path of the item.
    let full_path = get_path_from_item(model, item, true);

    // Get the times we must to go up until we reach the parent.
    let cycles = if full_path.len() > 0 { full_path.len() - 1 } else { 0 };

    // Get his parent.
    let mut parent;
    unsafe { parent = item.as_mut().unwrap().parent(); }

    // Unleash hell upon the land.
    for _ in 0..cycles {

        let childs;
        unsafe { childs = parent.as_mut().unwrap().row_count(); }
        for child in 0..childs {
            let item;
            let colour;
            unsafe { item = parent.as_mut().unwrap().child(child); }
            unsafe { colour = item.as_mut().unwrap().background().color().name(()).to_std_string(); }

            // If it's not transparent, stop.
            if colour != "#000000" { return }
        }

        // If no childs were modified, change the parent and try again.
        unsafe { parent.as_mut().unwrap().set_background(&Brush::new(GlobalColor::Transparent)); }
        unsafe { parent = parent.as_mut().unwrap().parent(); }
    }

    // If no more files were modified, set the mod as "not modified".
    set_modified(false, app_ui, None);
}

/// This function deletes whatever it's in the right side of the screen, leaving it empty.
/// Also, each time this triggers we consider there is no PackedFile open.
pub fn purge_them_all(app_ui: &AppUI, packedfiles_open_in_packedfile_view: &Rc<RefCell<BTreeMap<i32, Rc<RefCell<Vec<String>>>>>>) {

    // Black magic.
    unsafe {
        for item in 0..app_ui.packed_file_splitter.as_mut().unwrap().count() {
            let child = app_ui.packed_file_splitter.as_mut().unwrap().widget(item);
            child.as_mut().unwrap().hide();
            (child as *mut Object).as_mut().unwrap().delete_later();
        }
    }

    // Set it as not having an opened PackedFile, just in case.
    packedfiles_open_in_packedfile_view.borrow_mut().clear();

    // Just in case what was open before this was a DB Table, make sure the "Game Selected" menu is re-enabled.
    unsafe { app_ui.game_selected_group.as_mut().unwrap().set_enabled(true); }
}

/// This function deletes whatever it's in the specified position of the right side of the screen.
/// Also, if there was a PackedFile open there, we remove it from the "open PackedFiles" list.
pub fn purge_that_one_specifically(app_ui: &AppUI, the_one: i32, packedfiles_open_in_packedfile_view: &Rc<RefCell<BTreeMap<i32, Rc<RefCell<Vec<String>>>>>>) {

    // Turns out that deleting an item alters the order of the other items, so we schedule it for deletion, then put
    // an invisible item in his place. That does the job.
    unsafe {
        if app_ui.packed_file_splitter.as_mut().unwrap().count() > the_one {        
            let child = app_ui.packed_file_splitter.as_mut().unwrap().widget(the_one);
            child.as_mut().unwrap().hide();
            (child as *mut Object).as_mut().unwrap().delete_later();
        }
    }

    let widget = Widget::new().into_raw();
    unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(the_one, widget); }
    unsafe { widget.as_mut().unwrap().hide(); }

    // Set it as not having an opened PackedFile, just in case.
    packedfiles_open_in_packedfile_view.borrow_mut().remove(&the_one);

    // Just in case what was open before this was a DB Table, make sure the "Game Selected" menu is re-enabled.
    let mut x = false;
    for packed_file in packedfiles_open_in_packedfile_view.borrow().values() {
        if let Some(folder) = packed_file.borrow().get(0) {
            if folder == "db" {
                x = true;
                break;
            }
        }
    }

    if !x { unsafe { app_ui.game_selected_group.as_mut().unwrap().set_enabled(true); }}
}


/// This function shows the tips in the PackedFile View. Remember to call "purge_them_all" before this!
pub fn display_help_tips(app_ui: &AppUI) {

    // Create the widget that'll act as a container for the view.
    let widget = Widget::new().into_raw();
    let widget_layout = GridLayout::new().into_raw();
    unsafe { widget.as_mut().unwrap().set_layout(widget_layout as *mut Layout); }
    unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(0, widget); }

    let label = Label::new(&QString::from_std_str("Welcome to Rusted PackFile Manager! Here you have some tips on how to use it:
    - Read the manual. It's in 'About/Open Manual'. It explains how to configure RPFM and how to use it.
    - To know what each option in 'Preferences' do, left the mouse over the option for one second and a tooltip will pop up.
    - In the 'About' Menu, in 'About RPFM' you can find links to the Source Code and the Patreon of the Project.")).into_raw();

    unsafe { widget_layout.as_mut().unwrap().add_widget((label as *mut Widget, 0, 0, 1, 1)); }
}

/// This function shows a message asking for confirmation. For use in operations that implies unsaved
/// data loss. is_modified = true for when you can lose unsaved changes, is_delete_my_mod = true for
/// the deletion warning of MyMods.
pub fn are_you_sure(
    app_ui: &AppUI,
    is_modified: &Rc<RefCell<bool>>,
    is_delete_my_mod: bool
) -> bool {

    // If the mod has been modified...
    if *is_modified.borrow() {

        // Create the dialog.
        let mut dialog;
        unsafe { dialog = MessageBox::new_unsafe((
            &QString::from_std_str("Rusted PackFile Manager"),
            &QString::from_std_str("<p>There are some changes yet to be saved.</p><p>Are you sure?</p>"),
            Icon::Warning,
            65536, // No
            16384, // Yes
            1, // By default, select yes.
            app_ui.window as *mut Widget,
        )); }

        // Run the dialog and get the response. Yes => 3, No => 4.
        if dialog.exec() == 3 { true } else { false }
    }

    // If we are going to delete a MyMod...
    else if is_delete_my_mod {

        // Create the dialog.
        let mut dialog;
        unsafe { dialog = MessageBox::new_unsafe((
            &QString::from_std_str("Rusted PackFile Manager"),
            &QString::from_std_str("<p>You are about to delete this <i>'MyMod'</i> from your disk.</p><p>There is no way to recover it after that.</p><p>Are you sure?</p>"),
            Icon::Warning,
            65536, // No
            16384, // Yes
            1, // By default, select yes.)
            app_ui.window as *mut Widget,
        )); }

        // Run the dialog and get the response. Yes => 3, No => 4.
        if dialog.exec() == 3 { true } else { false }
    }

    // Otherwise, we allow the change directly.
    else { true }
}

/// This function is used to expand the entire path from the PackFile to an specific item in the TreeView.
pub fn expand_treeview_to_item(
    tree_view: *mut TreeView,
    model: *mut StandardItemModel,
    path: &[String],
) {
    // Get it another time, this time to use it to hold the current item.
    let mut item;
    unsafe { item = model.as_ref().unwrap().item(0); }
    unsafe { tree_view.as_mut().unwrap().expand(&model.as_ref().unwrap().index_from_item(item)); }

    // Indexes to see how deep we must go.
    let mut index = 0;
    let path_deep = path.len();

    // First looping downwards.
    loop {

        // If we reached the folder of the file, stop.
        if index == (path_deep - 1) { return }

        // If we are not still in the folder of the file...
        else {

            // Get the amount of children of the current item.
            let children_count;
            unsafe { children_count = item.as_ref().unwrap().row_count(); }

            // Bool to know when to stop in case of not finding the path.
            let mut not_found = true;

            // For each children we have...
            for row in 0..children_count {

                // Check if it has children of his own.
                let child;
                let has_children;
                unsafe { child = item.as_ref().unwrap().child(row); }
                unsafe { has_children = child.as_ref().unwrap().has_children(); }

                // If it doesn't have children, continue with the next child.
                if !has_children { continue; }

                // Get his text.
                let text;
                unsafe { text = child.as_ref().unwrap().text().to_std_string(); }

                // If it's the one we're looking for...
                if text == path[index] {

                    // Use it as our new item.
                    item = child;

                    // Increase the index.
                    index += 1;

                    // Tell the progam you found the child.
                    not_found = false;

                    // Expand the folder.
                    unsafe { tree_view.as_mut().unwrap().expand(&model.as_ref().unwrap().index_from_item(item)); }

                    // Break the loop.
                    break;
                }
            }

            // If the child was not found, stop and return the parent.
            if not_found { break; }
        }
    }
}

/// This function is used to get the complete Path of a Selected Item in the TreeView.
/// Set include_bool to true to include the PackFile in the path (like it's in the TreeView).
/// If you want to use your own model and selection, use "get_path_from_item_selection" instead.
pub fn get_path_from_selection(
    app_ui: &AppUI,
    include_packfile: bool
) -> Vec<String> {

    // Create the vector to hold the Path.
    let mut path: Vec<String> = vec![];

    // Get the selection of the TreeView.
    let selection_model;
    let mut selection;
    unsafe { selection_model = app_ui.folder_tree_view.as_mut().unwrap().selection_model(); }
    unsafe { selection = selection_model.as_mut().unwrap().selected_indexes(); }

    // If the selection has something...
    if selection.count(()) > 0 {

        // Get the selected cell.
        let mut item = selection.take_at(0);
        let mut parent;

        // Loop until we reach the root index.
        loop {

            // Get his data.
            let name;
            unsafe { name = app_ui.folder_tree_model.as_mut().unwrap().data(&item).to_string().to_std_string(); }

            // Add it to the list
            path.push(name);

            // Get the Parent of the item.
            parent = item.parent();

            // If the parent is valid, it's the new item.
            if parent.is_valid() { item = parent; }

            // Otherwise, we stop.
            else { break; }
        }

        // If we don't want to include the PackFile in the Path, remove it.
        if !include_packfile { path.pop(); }

        // Reverse it, as we want it from Parent to Children.
        path.reverse();

        // Return the Path.
        path
    }

    // Otherwise, we return an empty path.
    else { path }
}

/// This function is used to get the complete Path of a Selected Item in a StandardItemModel.
/// Set include_bool to true to include the PackFile in the path (like it's in the TreeView).
/// If you want to get the selection from the Main TreeView, use "get_path_from_selection" instead.
pub fn get_path_from_item_selection(
    model: *mut StandardItemModel,
    item: &ItemSelection,
    include_packfile: bool
) -> Vec<String>{

    // Create the vector to hold the Path.
    let mut path: Vec<String> = vec![];

    // Get the selection of the TreeView.
    let mut selection = item.indexes();

    // If the selection has something...
    if selection.count(()) > 0 {

        // Get the selected cell.
        let mut item = selection.take_at(0);
        let mut parent;

        // Loop until we reach the root index.
        loop {

            // Get his data.
            let name;
            unsafe { name = model.as_mut().unwrap().data(&item).to_string().to_std_string(); }

            // Add it to the list
            path.push(name);

            // Get the Parent of the item.
            parent = item.parent();

            // If the parent is valid, it's the new item.
            if parent.is_valid() { item = parent; }

            // Otherwise, we stop.
            else { break; }
        }

        // If we don't want to include the PackFile in the Path, remove it.
        if !include_packfile { path.pop(); }

        // Reverse it, as we want it from Parent to Children.
        path.reverse();

        // Return the Path.
        path
    }

    // Otherwise, return an empty path.
    else { path }
}

/// This function is used to get the complete Path of a specific Item in a StandardItemModel.
/// Set include_bool to true to include the PackFile in the path (like it's in the TreeView).
pub fn get_path_from_item(
    model: *mut StandardItemModel,
    item_raw: *mut StandardItem,
    include_packfile: bool
) -> Vec<String>{

    // Create the vector to hold the Path.
    let mut path: Vec<String> = vec![];

    // Get the item of the TreeView.
    let mut item;
    let mut parent;
    unsafe { item = item_raw.as_mut().unwrap().index(); }

    // Loop until we reach the root index.
    loop {

        // Get his data.
        let name;
        unsafe { name = model.as_mut().unwrap().data(&item).to_string().to_std_string(); }

        // Add it to the list
        path.push(name);

        // Get the Parent of the item.
        parent = item.parent();

        // If the parent is valid, it's the new item.
        if parent.is_valid() { item = parent; }

        // Otherwise, we stop.
        else { break; }
    }

    // If we don't want to include the PackFile in the Path, remove it.
    if !include_packfile { path.pop(); }

    // Reverse it, as we want it from Parent to Children.
    path.reverse();

    // Return the Path.
    path
}

/// This function is used to get the path it'll have in the TreeView an File/Folder from the FileSystem.
/// is_file = true should be set in case we want to know the path of a file. Otherwise, the function will
/// treat the Item from the FileSystem as a folder.
pub fn get_path_from_pathbuf(
    app_ui: &AppUI,
    file_path: &PathBuf,
    is_file: bool
) -> Vec<Vec<String>> {

    // Create the vector to hold the Path.
    let mut paths: Vec<Vec<String>> = vec![];

    // If it's a single file, we get his name and push it to the paths vector.
    if is_file { paths.push(vec![file_path.file_name().unwrap().to_string_lossy().as_ref().to_owned()]); }

    // Otherwise, it's a folder, so we have to filter it first.
    else {

        // Get the "Prefix" of the folder (path without the folder's name).
        let mut useless_prefix = file_path.to_path_buf();
        useless_prefix.pop();

        // Get the paths of all the files inside that folder, recursively.
        let file_list = get_files_from_subdir(&file_path).unwrap();

        // Then, for each file...
        for file_path in &file_list {

            // Remove his prefix, leaving only the path from the folder onwards.
            let filtered_path = file_path.strip_prefix(&useless_prefix).unwrap();

            // Turn it from &Path to a Vec<String>, reverse it, and push it to the list.
            let mut filtered_path = filtered_path.iter().map(|x| x.to_string_lossy().as_ref().to_owned()).collect::<Vec<String>>();
            filtered_path.reverse();
            paths.push(filtered_path);
        }
    }

    // For each path we have...
    for path in &mut paths {

        // Get his base path without the PackFile.
        let mut base_path = get_path_from_selection(&app_ui, false);

        // Combine it with his path to form his full form.
        base_path.reverse();
        path.append(&mut base_path);
        path.reverse();
    }

    // Return the paths (sorted from parent to children)
    paths
}

/// This function gets you the StandardItem corresponding to a certain path in a TreeView. It uses a path without PackFile.
pub fn get_item_from_incomplete_path(
    model: *mut StandardItemModel,
    path: &[String],
) -> *mut StandardItem {

    // Get it another time, this time to use it to hold the current item.
    let mut item;
    unsafe { item = model.as_ref().unwrap().item(0); }

    // Indexes to see how deep we must go.
    let mut index = 0;
    let path_deep = path.len();

    // First looping downwards.
    loop {

        // If we reached the folder of the file...
        if index == (path_deep - 1) {

            // Get the amount of children of the current item.
            let children_count;
            unsafe { children_count = item.as_ref().unwrap().row_count(); }

            // For each children we have...
            for row in 0..children_count {

                // Check if it has children of his own.
                let child;
                let has_children;
                unsafe { child = item.as_ref().unwrap().child(row); }
                unsafe { has_children = child.as_ref().unwrap().has_children(); }

                // If has children, continue with the next child.
                if has_children { continue; }

                // Get his text.
                let text;
                unsafe { text = child.as_ref().unwrap().text().to_std_string(); }

                // TODO: This can crash. Fix it properly.
                // If it's the one we're looking for...
                if text == path[index] {

                    // Use it as our new item.
                    item = child;

                    // And break the loop.
                    break;
                }
            }

            // End the first loop.
            break;
        }

        // If we are not still in the folder of the file...
        else {

            // Get the amount of children of the current item.
            let children_count;
            unsafe { children_count = item.as_ref().unwrap().row_count(); }

            // Bool to know when to stop in case of not finding the path.
            let mut not_found = true;

            // For each children we have...
            for row in 0..children_count {

                // Check if it has children of his own.
                let child;
                let has_children;
                unsafe { child = item.as_ref().unwrap().child(row); }
                unsafe { has_children = child.as_ref().unwrap().has_children(); }

                // If it doesn't have children, continue with the next child.
                if !has_children { continue; }

                // Get his text.
                let text;
                unsafe { text = child.as_ref().unwrap().text().to_std_string(); }

                // If it's the one we're looking for...
                if text == path[index] {

                    // Use it as our new item.
                    item = child;

                    // Increase the index.
                    index += 1;

                    // Tell the progam you found the child.
                    not_found = false;

                    // Break the loop.
                    break;
                }
            }

            // If the child was not found, stop and return the parent.
            if not_found { break; }
        }
    }

    // Return the item.
    item
}

/// This function paints the entire path to it, depending on if it's a modification or an addition.
/// This requires the item to be in the Model already. Otherwise it'll not work.
pub fn paint_treeview(
    item: *mut StandardItem,
    model: *mut StandardItemModel,
    status: ItemVisualStatus,
) {

    // Get the colors we need to apply.
    let color_added = if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkGreen } else { GlobalColor::Green };
    let color_modified = if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkYellow } else { GlobalColor::Yellow };
    let color_added_modified = if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkMagenta } else { GlobalColor::Magenta };
    let color_untouched = GlobalColor::Transparent;
    let color = match &status {
        ItemVisualStatus::Added => color_added,
        ItemVisualStatus::Modified => color_modified,
        ItemVisualStatus::AddedModified => color_added_modified.clone(),
        ItemVisualStatus::Untouched => color_untouched,
    };

    // Get the full path of the item and the times we must to go up until we reach the parent.
    let full_path = get_path_from_item(model, item, true);
    let cycles = if full_path.len() > 0 { full_path.len() - 1 } else { 0 };

    // Paint it like one of your french girls.
    unsafe { item.as_mut().unwrap().set_background(&Brush::new(color.clone())); }

    // Loop through his parents until we reach the PackFile
    let mut parent = unsafe { item.as_mut().unwrap().parent() };
    for _ in 0..cycles {

        // Get the status of the Parent depending on his color.
        let parent_color = unsafe { parent.as_mut().unwrap().background().color().name(()).to_std_string() };
        let parent_status = match &*parent_color {
            "#00ff00" | "800000" => ItemVisualStatus::Added,
            "#ffff00" | "808000" => ItemVisualStatus::Modified,
            "#ff00ff" | "800080" => ItemVisualStatus::AddedModified,
            "#000000" | _ => ItemVisualStatus::Untouched,
        };

        // Paint it depending on his status.
        match parent_status {

            // If it's Added and the new status is "Modified", turn it into "AddedModified".
            ItemVisualStatus::Added => {
                if status == ItemVisualStatus::Modified { unsafe { parent.as_mut().unwrap().set_background(&Brush::new(color_added_modified.clone())); } }
            },

            // If it's Modified and the new status is "Added", turn it into "AddedModified".
            ItemVisualStatus::Modified => {
                if status == ItemVisualStatus::Added { unsafe { parent.as_mut().unwrap().set_background(&Brush::new(color_added_modified.clone())); } }
            },

            // If it's AddedModified, left it as is.
            ItemVisualStatus::AddedModified => {},

            // If it doesn't had an state before, apply the same as the child.
            ItemVisualStatus::Untouched => unsafe { parent.as_mut().unwrap().set_background(&Brush::new(color.clone())); }
        }

        // Set the new parent.
        unsafe { parent = parent.as_mut().unwrap().parent(); }
    }
}

/// This function cleans the entire TreeView from colors. To be used when saving.
pub fn clean_treeview(
    item: *mut StandardItem,
    model: *mut StandardItemModel
) {

    // Get the color we need to apply.
    let color = GlobalColor::Transparent;

    // Paint the current item.
    unsafe { item.as_mut().unwrap().set_background(&Brush::new(color)); }

    // Get the amount of children of the current item.
    let children_count;
    unsafe { children_count = item.as_ref().unwrap().row_count(); }

    // For each children we have...
    for row in 0..children_count {

        // Get the child.
        let child;
        unsafe { child = item.as_ref().unwrap().child(row); }
        
        // Paint him and his children too.
        clean_treeview(child, model);

    }
}

/// This function is used to set the icon of an Item in the TreeView. It requires:
/// - item: the item to put the icon in.
/// - icons: the list of pre-generated icons.
/// - icon_type: the type of icon needed for this file.
fn set_icon_to_item(
    item: *mut StandardItem,
    icon_type: IconType,
) {

    // Depending on the IconType we receive...
    match icon_type {

        // For PackFiles.
        IconType::PackFile(editable) => {
            if editable { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.packfile_editable); } }
            else { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.packfile_locked); } }
        },

        // For folders.
        IconType::Folder => unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.folder); },

        // For files.
        IconType::File(path) => {

            // Get the name of the file.
            let packed_file_name = path.last().unwrap();

            // If it's in the "db" folder, it's a DB PackedFile (or you put something were it shouldn't be).
            if path[0] == "db" { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.table); } }

            // If it ends in ".loc", it's a localisation PackedFile.
            else if packed_file_name.ends_with(".loc") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.table); } }

            // If it ends in ".rigid_model_v2", it's a RigidModel PackedFile.
            else if packed_file_name.ends_with(".rigid_model_v2") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.rigid_model); } }

            // If it ends in any of these, it's a plain text PackedFile.
            else if packed_file_name.ends_with(".lua") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_generic); } }
            else if packed_file_name.ends_with(".xml") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_xml); } }
            else if packed_file_name.ends_with(".xml.shader") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_xml); } }
            else if packed_file_name.ends_with(".xml.material") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_xml); } }
            else if packed_file_name.ends_with(".variantmeshdefinition") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_xml); } }
            else if packed_file_name.ends_with(".environment") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_xml); } }
            else if packed_file_name.ends_with(".lighting") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_generic); } }
            else if packed_file_name.ends_with(".wsmodel") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_generic); } }
            else if packed_file_name.ends_with(".csv") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_csv); } }
            else if packed_file_name.ends_with(".tsv") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_csv); } }
            else if packed_file_name.ends_with(".inl") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_generic); } }
            else if packed_file_name.ends_with(".battle_speech_camera") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_generic); } }
            else if packed_file_name.ends_with(".bob") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_generic); } }
            else if packed_file_name.ends_with(".cindyscene") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_generic); } }
            else if packed_file_name.ends_with(".cindyscenemanager") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_generic); } }
            //else if packed_file_name.ends_with(".benchmark") || // This one needs special decoding/encoding.
            else if packed_file_name.ends_with(".txt") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_txt); } }

            // If it ends in any of these, it's an image.
            else if packed_file_name.ends_with(".jpg") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.image_jpg); } }
            else if packed_file_name.ends_with(".jpeg") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.image_jpg); } }
            else if packed_file_name.ends_with(".tga") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.image_generic); } }
            else if packed_file_name.ends_with(".dds") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.image_generic); } }
            else if packed_file_name.ends_with(".png") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.image_png); } }

            // Otherwise, it's a generic file.
            else { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.file); } }
        }
    }
}

/// This function takes care of EVERY operation that manipulates the provided TreeView.
/// It does one thing or another, depending on the operation we provide it.
pub fn update_treeview(
    sender_qt: &Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt_data: Rc<RefCell<Receiver<Data>>>,
    window: *mut MainWindow,
    tree_view: *mut TreeView,
    model: *mut StandardItemModel,
    operation: TreeViewOperation,
) {

    // We act depending on the operation requested.
    match operation {

        // If we want to build a new TreeView...
        TreeViewOperation::Build(is_extra_packfile) => {

            // Depending on the PackFile we want to build the TreeView with, we ask for his data.
            if is_extra_packfile { sender_qt.send(Commands::GetPackFileExtraDataForTreeView).unwrap(); }
            else { sender_qt.send(Commands::GetPackFileDataForTreeView).unwrap(); }
            let pack_file_data = if let Data::StringI64VecVecString(data) = check_message_validity_recv2(&receiver_qt_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

            // First, we clean the TreeStore and whatever was created in the TreeView.
            unsafe { model.as_mut().unwrap().clear(); }

            // Second, we set as the big_parent, the base for the folders of the TreeView, a fake folder
            // with the name of the PackFile. All big things start with a lie.
            let mut big_parent = StandardItem::new(&QString::from_std_str(pack_file_data.0)).into_raw();

            // Get his last modified date and show it in a tooltip.
            unsafe { big_parent.as_mut().unwrap().set_tool_tip(&QString::from_std_str(format!("Last Modified: {:?}", NaiveDateTime::from_timestamp(pack_file_data.1, 0)))); }

            // Also, set it as not editable by the user. Otherwise will cause problems when renaming.
            unsafe { big_parent.as_mut().unwrap().set_editable(false); }

            // Add the Big Parent to the Model.
            unsafe { model.as_mut().unwrap().append_row_unsafe(big_parent); }

            // Give it an Icon.
            set_icon_to_item(big_parent, IconType::PackFile(is_extra_packfile));

            // We get all the paths of the PackedFiles inside the Packfile in a Vector.
            let mut sorted_path_list = pack_file_data.2;

            // We sort that vector using this horrific monster I don't want to touch again, using
            // the following format:
            // - FolderA
            // - FolderB
            // - FileA
            // - FileB
            sorted_path_list.sort_unstable_by(|a, b| {
                let mut index = 0;
                loop {

                    // If both options have the same name.
                    if a[index] == b[index] {

                        // If A doesn't have more children, but B has them, A is a file and B a folder.
                        if index == (a.len() - 1) && index < (b.len() - 1) {
                            return Ordering::Greater
                        }

                        // If B doesn't have more children, but A has them, B is a file and A a folder.
                        else if index < (a.len() - 1) && index == (b.len() - 1) {
                            return Ordering::Less
                        }

                        // If both options still has children, continue the loop.
                        else if index < (a.len() - 1) && index < (b.len() - 1) {
                            index += 1;
                            continue;
                        }
                    }
                    // If both options have different name,...
                    // If both are the same type (both have children, or none have them), doesn't matter if
                    // they are files or folder. Just compare them to see what one it's first.
                    else if (index == (a.len() - 1) && index == (b.len() - 1)) ||
                        (index < (a.len() - 1) && index < (b.len() - 1)) {
                        return a.cmp(b)
                    }

                    // If A doesn't have more children, but B has them, A is a file and B a folder.
                    else if index == (a.len() - 1) && index < (b.len() - 1) {
                        return Ordering::Greater

                    }
                    // If B doesn't have more children, but A has them, B is a file and A a folder.
                    else if index < (a.len() - 1) && index == (b.len() - 1) {
                        return Ordering::Less
                    }
                }
            });

            // Once we get the entire path list sorted, we add the paths to the model one by one,
            // skipping duplicate entries.
            for path in &sorted_path_list {

                // First, we reset the parent to the big_parent (the PackFile).
                let mut parent;
                unsafe { parent = model.as_ref().unwrap().item(0); }

                // Then, we form the path ("parent -> child" style path) to add to the model.
                for name in path.iter() {

                    // If it's the last string in the file path, it's a file, so we add it to the model.
                    if name == path.last().unwrap() {

                        // Create the item.
                        let mut file = StandardItem::new(&QString::from_std_str(name)).into_raw();

                        // Set it as not editable by the user. Otherwise will cause problems when renaming.
                        unsafe { file.as_mut().unwrap().set_editable(false); }

                        // Add it to the TreeView.
                        unsafe { parent.as_mut().unwrap().append_row_unsafe(file); }

                        // Get the Path of the File.
                        let path = get_path_from_item(model, file, false);

                        // Give it an icon.
                        set_icon_to_item(file, IconType::File(path));
                    }

                    // If it's a folder, we check first if it's already in the TreeStore using the following
                    // logic:
                    // If the current parent has a child, it should be a folder already in the TreeStore,
                    // so we check all his children. If any of them is equal to the current folder we are
                    // trying to add and it has at least one child, it's a folder exactly like the one we are
                    // trying to add, so that one becomes our new parent. If there is no equal folder to
                    // the one we are trying to add, we add it, turn it into the new parent, and repeat.
                    else {

                        // There are many unsafe things in this code...
                        unsafe {

                            // Variable to check if the current folder is already in the TreeView.
                            let mut duplicate_found = false;

                            // If the current parent has at least one child...
                            if parent.as_ref().unwrap().has_children() {

                                // It's a folder, so we check his children.
                                for index in 0..parent.as_ref().unwrap().row_count() {

                                    // Get the child.
                                    let mut child = parent.as_mut().unwrap().child((index, 0));

                                    // Get his text.
                                    let child_text = child.as_ref().unwrap().text().to_std_string();

                                    // If it's the same folder we are trying to add...
                                    if child_text == *name {

                                        // This is our parent now.
                                        parent = parent.as_mut().unwrap().child(index);
                                        duplicate_found = true;
                                        break;
                                    }
                                }

                                // If we found a duplicate, skip to the next file/folder.
                                if duplicate_found { continue; }

                                // Otherwise, add it to the parent, and turn it into the new parent.
                                else {

                                    // Create the item.
                                    let mut folder = StandardItem::new(&QString::from_std_str(name)).into_raw();

                                    // Set it as not editable by the user. Otherwise will cause problems when renaming.
                                    folder.as_mut().unwrap().set_editable(false);

                                    // Add it to the model.
                                    parent.as_mut().unwrap().append_row_unsafe(folder);

                                    // Give it an Icon.
                                    set_icon_to_item(folder, IconType::Folder);

                                    // This is our parent now.
                                    let index = parent.as_ref().unwrap().row_count() - 1;
                                    parent = parent.as_mut().unwrap().child(index);
                                }
                            }

                            // If our current parent doesn't have anything, just add it.
                            else {

                                // Create the item.
                                let mut folder = StandardItem::new(&QString::from_std_str(name)).into_raw();

                                // Set it as not editable by the user. Otherwise will cause problems when renaming.
                                folder.as_mut().unwrap().set_editable(false);

                                // Add it to the model.
                                parent.as_mut().unwrap().append_row_unsafe(folder);

                                // Give it an Icon.
                                set_icon_to_item(folder, IconType::Folder);

                                // This is our parent now.
                                let index = parent.as_ref().unwrap().row_count() - 1;
                                parent = parent.as_mut().unwrap().child(index);
                            }
                        }
                    }
                }
            }
        },

        // If we want to add a file/folder to the `TreeView`...
        TreeViewOperation::Add(paths) => {

            // For each path in our list of paths to add...
            for path in &paths {

                // First, we get the item of our PackFile in the TreeView.
                let mut parent;
                unsafe { parent = model.as_ref().unwrap().item(0); }

                // For each field in our path...
                for (index, field) in path.iter().enumerate() {

                    // If it's the last one of the path, it's a file.
                    if index >= (path.len() - 1) {

                        // Try to get an item from this path.
                        let possible_item = get_item_from_incomplete_path(model, path);

                        // Try to get the path from that item.
                        let possible_path = get_path_from_item(model, possible_item, false);

                        // If the path already exists, it means we have overwritten his file, so consider it as good as new.
                        if &possible_path == path {

                            // Just re-paint it like that parrot you painted yesterday.
                            paint_treeview(possible_item, model, ItemVisualStatus::Added);
                        }

                        // Otherwise, it's a new PackedFile, so do the usual stuff.
                        else {

                            // Add the file to the TreeView.
                            let item = StandardItem::new(&QString::from_std_str(field)).into_raw();

                            // Also, set it as not editable by the user. Otherwise will cause problems when renaming.
                            unsafe { item.as_mut().unwrap().set_editable(false); }
                            unsafe { parent.as_mut().unwrap().append_row_unsafe(item); }

                            // Get the Path of the File.
                            let path = get_path_from_item(model, item, true);

                            // Send the Path to the Background Thread to get the Item's Type.
                            sender_qt.send(Commands::GetTypeOfPath).unwrap();
                            sender_qt_data.send(Data::VecString(path)).unwrap();
                            let item_type = if let Data::TreePathType(data) = check_message_validity_recv2(&receiver_qt_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                            // Act depending on the Type of the Path.
                            match item_type {

                                // If it's a folder, give it an Icon.
                                TreePathType::Folder(_) => set_icon_to_item(item, IconType::Folder),

                                // If it's a folder, give it an Icon.
                                TreePathType::File(ref path) => set_icon_to_item(item, IconType::File(path.to_vec())),

                                // Any other type, ignore it.
                                _ => {},
                            }

                            // Paint it like that parrot you painted yesterday.
                            paint_treeview(item, model, ItemVisualStatus::Added);

                            // Sort the TreeView.
                            sort_item_in_tree_view(
                                sender_qt,
                                sender_qt_data,
                                receiver_qt_data.clone(),
                                model,
                                item,
                                item_type
                            );
                        }
                    }

                    // Otherwise, it's a folder.
                    else {

                        unsafe {

                            // If the current parent has at least one child...
                            if parent.as_ref().unwrap().has_children() {

                                // Variable to check if the current folder is already in the TreeView.
                                let mut duplicate_found = false;

                                // It's a folder, so we check his children.
                                for index in 0..parent.as_ref().unwrap().row_count() {

                                    // Get the child.
                                    let mut child = parent.as_mut().unwrap().child((index, 0));

                                    // Get his text.
                                    let child_text = child.as_ref().unwrap().text().to_std_string();

                                    // If it's the same folder we are trying to add...
                                    if child_text == *field {

                                        // This is our parent now.
                                        parent = parent.as_mut().unwrap().child(index);
                                        duplicate_found = true;
                                        break;
                                    }
                                }

                                // If we found a duplicate, skip to the next file/folder.
                                if duplicate_found { continue; }

                                // Otherwise, add it to the parent, and turn it into the new parent.
                                else {

                                    // Create the item.
                                    let mut folder = StandardItem::new(&QString::from_std_str(field)).into_raw();

                                    // Set it as not editable by the user. Otherwise will cause problems when renaming.
                                    folder.as_mut().unwrap().set_editable(false);
                                    parent.as_mut().unwrap().append_row_unsafe(folder);

                                    // Give it an icon.
                                    set_icon_to_item(folder, IconType::Folder);

                                    // This is our parent now.
                                    let index = parent.as_ref().unwrap().row_count() - 1;
                                    parent = parent.as_mut().unwrap().child(index);

                                    // Sort the TreeView.
                                    sort_item_in_tree_view(
                                        sender_qt,
                                        sender_qt_data,
                                        receiver_qt_data.clone(),
                                        model,
                                        folder,
                                        TreePathType::Folder(vec![String::new()])
                                    );
                                }
                            }

                            // If our current parent doesn't have anything, just add it.
                            else {

                                // Create the Item.
                                let mut folder = StandardItem::new(&QString::from_std_str(field)).into_raw();

                                // Set it as not editable by the user. Otherwise will cause problems when renaming.
                                folder.as_mut().unwrap().set_editable(false);
                                parent.as_mut().unwrap().append_row_unsafe(folder);

                                // Give it an icon.
                                set_icon_to_item(folder, IconType::Folder);

                                // This is our parent now.
                                let index = parent.as_ref().unwrap().row_count() - 1;
                                parent = parent.as_mut().unwrap().child(index);

                                // Sort the TreeView.
                                sort_item_in_tree_view(
                                    sender_qt,
                                    sender_qt_data,
                                    receiver_qt_data.clone(),
                                    model,
                                    folder,
                                    TreePathType::Folder(vec![String::new()])
                                );
                            }
                        }
                    }
                }
            }
        },

        // If we want to delete something selected from the `TreeView`...
        TreeViewOperation::DeleteSelected(path_type) => {

            // Then we see what type the selected thing is.
            match path_type {

                // If it's a PackedFile or a Folder...
                TreePathType::File(_) | TreePathType::Folder(_) => {

                    // Get whatever is selected from the TreeView.
                    let packfile;
                    let selection_model;
                    let mut selection;
                    unsafe { selection_model = tree_view.as_mut().unwrap().selection_model(); }
                    unsafe { selection = selection_model.as_mut().unwrap().selected_indexes(); }
                    unsafe { packfile = model.as_ref().unwrap().item(0); }
                    let mut item = selection.take_at(0);
                    let mut parent;

                    // Begin the endless cycle of war and dead.
                    loop {

                        // Get the parent of the item.
                        parent = item.parent();

                        // Kill the item in a cruel way.
                        unsafe { model.as_mut().unwrap().remove_row((item.row(), &parent));}

                        // Check if the parent still has children.
                        let has_children;
                        let packfile_has_children;
                        unsafe { has_children = model.as_mut().unwrap().has_children(&parent); }
                        unsafe { packfile_has_children = packfile.as_ref().unwrap().has_children(); }

                        // If the parent has more children, or we reached the PackFile, we're done.
                        if has_children | !packfile_has_children { break; }

                        // Otherwise, our new item is our parent.
                        else { item = parent }
                    }
                }

                // If it's a PackFile...
                TreePathType::PackFile => {

                    // Rebuild the TreeView.
                    update_treeview(
                        &sender_qt,
                        &sender_qt_data,
                        receiver_qt_data.clone(),
                        window,
                        tree_view,
                        model,
                        TreeViewOperation::Build(false),
                    );
                },

                // If we don't have anything selected, we do nothing.
                TreePathType::None => {},
            }
        },

        // If we want to delete something from the TreeView, independant of his selection...
        TreeViewOperation::DeleteUnselected(path_type) => {

            // Then we see what type the selected thing is.
            match path_type {

                // If it's a PackedFile or a Folder...
                TreePathType::File(path) => {

                    // Get the PackFile's item.
                    let packfile;
                    unsafe { packfile = model.as_ref().unwrap().item(0); }

                    // Get it another time, this time to use it to hold the current item.
                    let mut item;
                    unsafe { item = model.as_ref().unwrap().item(0); }

                    // Indexes to see how deep we must go.
                    let mut index = 0;
                    let path_deep = path.len();

                    // First looping downwards.
                    loop {

                        // If we reached the folder of the file...
                        if index == (path_deep - 1) {

                            // Get the amount of children of the current item.
                            let children_count;
                            unsafe { children_count = item.as_ref().unwrap().row_count(); }

                            // For each children we have...
                            for row in 0..children_count {

                                // Check if it has children of his own.
                                let child;
                                let has_children;
                                unsafe { child = item.as_ref().unwrap().child(row); }
                                unsafe { has_children = child.as_ref().unwrap().has_children(); }

                                // If has children, continue with the next child.
                                if has_children { continue; }

                                // Get his text.
                                let text;
                                unsafe { text = child.as_ref().unwrap().text().to_std_string(); }

                                // TODO: This can crash. Fix it properly.
                                // If it's the one we're looking for...
                                if text == path[index] {

                                    // Use it as our new item.
                                    item = child;

                                    // And break the loop.
                                    break;
                                }
                            }

                            // End the first loop.
                            break;
                        }

                        // If we are not still in the folder of the file...
                        else {

                            // Get the amount of children of the current item.
                            let children_count;
                            unsafe { children_count = item.as_ref().unwrap().row_count(); }

                            // For each children we have...
                            for row in 0..children_count {

                                // Check if it has children of his own.
                                let child;
                                let has_children;
                                unsafe { child = item.as_ref().unwrap().child(row); }
                                unsafe { has_children = child.as_ref().unwrap().has_children(); }

                                // If it doesn't have children, continue with the next child.
                                if !has_children { continue; }

                                // Get his text.
                                let text;
                                unsafe { text = child.as_ref().unwrap().text().to_std_string(); }

                                // If it's the one we're looking for...
                                if text == path[index] {

                                    // Use it as our new item.
                                    item = child;

                                    // Increase the index.
                                    index += 1;

                                    // Break the loop.
                                    break;
                                }
                            }
                        }
                    }

                    // Prepare the Parent...
                    let mut parent;

                    // Begin the endless cycle of war and dead.
                    loop {

                        // Get the parent of the item.
                        unsafe { parent = item.as_mut().unwrap().parent(); }

                        // Kill the item in a cruel way.
                        unsafe { parent.as_mut().unwrap().remove_row(item.as_mut().unwrap().row());}

                        // Check if the parent still has children.
                        let has_children;
                        let packfile_has_children;
                        unsafe { has_children = parent.as_mut().unwrap().has_children(); }
                        unsafe { packfile_has_children = packfile.as_ref().unwrap().has_children(); }

                        // If the parent has more children, or we reached the PackFile, we're done.
                        if has_children | !packfile_has_children { break; }

                        // Otherwise, our new item is our parent.
                        else { item = parent }
                    }
                }

                // If it's a PackFile...
                TreePathType::PackFile => {

                    // Get the name of the PackFile from the TreeView.
                    let packfile;
                    let name;
                    unsafe { packfile = model.as_ref().unwrap().item(0); }
                    unsafe { name = packfile.as_mut().unwrap().text(); }

                    // Clear the TreeModel.
                    unsafe { model.as_mut().unwrap().clear(); }

                    // Then we add the PackFile to it. This effectively deletes all the PackedFiles in the PackFile.
                    let mut pack_file = StandardItem::new(&name);
                    unsafe { model.as_mut().unwrap().append_row_unsafe(pack_file.into_raw()); }
                },

                // TODO: Implement this for folders.
                // If we don't have anything selected, we do nothing.
                _ => {},
            }
        },

        // If we want to rename something...
        TreeViewOperation::Rename(path_type, new_name) => {

            // Get the selection model.
            let selection_model;
            unsafe { selection_model = tree_view.as_mut().unwrap().selection_model(); }

            // Get the selected cell.
            let selection;
            unsafe { selection = selection_model.as_mut().unwrap().selected_indexes(); }
            let selection = selection.at(0);

            // Put the new name in a variant.
            let variant = Variant::new0(&QString::from_std_str(&new_name));

            // Change the old data with the new one.
            unsafe { model.as_mut().unwrap().set_data((selection, &variant)); }

            // Act depending on the Type of the Path.
            match path_type {

                // If it's a folder or a File, give it an Icon.
                TreePathType::Folder(_) | TreePathType::File(_) => {

                    // Get the item.
                    let item;
                    unsafe { item = model.as_mut().unwrap().item_from_index(selection); }

                    // Paint it as "modified".
                    paint_treeview(item, model, ItemVisualStatus::Modified);

                    // Sort it.
                    sort_item_in_tree_view(
                        sender_qt,
                        sender_qt_data,
                        receiver_qt_data.clone(),
                        model,
                        item,
                        path_type
                    );
                }

                // In any other case, don't do anything.
                _ => {},
            }
        },

        // If we want to apply a prefix to multiple files...
        TreeViewOperation::PrefixFiles(old_paths, prefix) => {

            // For each changed path...
            for path in old_paths {

                // Get the item and the new text.
                let mut item = get_item_from_incomplete_path(model, &path);
                let text;
                unsafe { text = item.as_mut().unwrap().text().to_std_string(); }
                let new_name = format!("{}{}", prefix, text); 
                unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str(&new_name)); }

                // Prepare the new path for the sorting function.
                let mut new_path = path.to_vec();
                new_path.pop();
                new_path.push(new_name);

                // Paint it as "modified".
                paint_treeview(item, model, ItemVisualStatus::Modified);

                // Sort it.
                sort_item_in_tree_view(
                    sender_qt,
                    sender_qt_data,
                    receiver_qt_data.clone(),
                    model,
                    item,
                    TreePathType::File(new_path)
                );
            }
        },
    }

    // If we have altered the TreeView in ANY way, we need to recheck the empty folders list.
    sender_qt.send(Commands::UpdateEmptyFolders).unwrap();
}

/// This function sorts items in a TreeView following this order:
/// - AFolder.
/// - aFolder.
/// - ZFolder.
/// - zFolder.
/// - AFile.
/// - aFile.
/// - ZFile.
/// - zFile.
/// The reason for this function is because the native Qt function doesn't order folders before files.
#[allow(dead_code)]
fn sort_item_in_tree_view(
    sender_qt: &Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt: Rc<RefCell<Receiver<Data>>>,
    model: *mut StandardItemModel,
    mut item: *mut StandardItem,
    item_type: TreePathType,
) {

    // Get the ModelIndex of our Item and his row, as that's what we are going to be changing.
    let mut item_index;
    unsafe { item_index = item.as_mut().unwrap().index(); }

    // Get the parent of the item.
    let parent;
    let parent_index;
    unsafe { parent = item.as_mut().unwrap().parent(); }
    unsafe { parent_index = parent.as_mut().unwrap().index(); }

    // Get the previous and next item ModelIndex on the list.
    let item_index_prev;
    let item_index_next;
    unsafe { item_index_prev = model.as_mut().unwrap().index((item_index.row() - 1, item_index.column(), &parent_index)); }
    unsafe { item_index_next = model.as_mut().unwrap().index((item_index.row() + 1, item_index.column(), &parent_index)); }

    // Get the type of the previous item on the list.
    let item_type_prev: TreePathType = if item_index_prev.is_valid() {

        // Get the previous item.
        let item_sibling;
        unsafe { item_sibling = model.as_mut().unwrap().item_from_index(&item_index_prev); }

        // Get the path of the previous item.
        let path = get_path_from_item(model, item_sibling, true);

        // Send the Path to the Background Thread, and get the type of the item.
        sender_qt.send(Commands::GetTypeOfPath).unwrap();
        sender_qt_data.send(Data::VecString(path)).unwrap();
        if let Data::TreePathType(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); }
    }

    // Otherwise, return the type as `None`.
    else { TreePathType::None };

    // Get the type of the previous and next items on the list.
    let item_type_next: TreePathType = if item_index_next.is_valid() {

        // Get the next item.
        let item_sibling;
        unsafe { item_sibling = model.as_mut().unwrap().item_from_index(&item_index_next); }

        // Get the path of the previous item.
        let path = get_path_from_item(model, item_sibling, true);

        // Send the Path to the Background Thread, and get the type of the item.
        sender_qt.send(Commands::GetTypeOfPath).unwrap();
        sender_qt_data.send(Data::VecString(path)).unwrap();
        if let Data::TreePathType(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); }
    }

    // Otherwise, return the type as `None`.
    else { TreePathType::None };

    // We get the boolean to determinate the direction to move (true -> up, false -> down).
    // If the previous and the next Items are `None`, we don't need to move.
    let direction = if item_type_prev == TreePathType::None && item_type_next == TreePathType::None { return }

    // If the top one is `None`, but the bottom one isn't, we go down.
    else if item_type_prev == TreePathType::None && item_type_next != TreePathType::None { false }

    // If the bottom one is `None`, but the top one isn't, we go up.
    else if item_type_prev != TreePathType::None && item_type_next == TreePathType::None { true }

    // If the top one is a folder, and the bottom one is a file, get the type of our iter.
    else if item_type_prev == TreePathType::Folder(vec![String::new()]) && item_type_next == TreePathType::File(vec![String::new()]) {
        if item_type == TreePathType::Folder(vec![String::new()]) { true } else { false }
    }

    // If the two around it are the same type, compare them and decide.
    else {

        // Get the previous, current and next texts.
        let previous_name: String;
        let current_name: String;
        let next_name: String;
        unsafe { previous_name = QString::to_std_string(&parent.as_mut().unwrap().child(item_index.row() - 1).as_mut().unwrap().text()); }
        unsafe { current_name = QString::to_std_string(&parent.as_mut().unwrap().child(item_index.row()).as_mut().unwrap().text()); }
        unsafe { next_name = QString::to_std_string(&parent.as_mut().unwrap().child(item_index.row() + 1).as_mut().unwrap().text()); }

        // If, after sorting, the previous hasn't changed position, it shouldn't go up.
        let name_list = vec![previous_name.to_owned(), current_name.to_owned()];
        let mut name_list_sorted = vec![previous_name.to_owned(), current_name.to_owned()];
        name_list_sorted.sort();
        if name_list == name_list_sorted {

            // If, after sorting, the next hasn't changed position, it shouldn't go down.
            let name_list = vec![current_name.to_owned(), next_name.to_owned()];
            let mut name_list_sorted = vec![current_name.to_owned(), next_name.to_owned()];
            name_list_sorted.sort();
            if name_list == name_list_sorted {

                // In this case, we don't move.
                return
            }

            // Go down.
            else { false }
        }

        // Go up.
        else { true }
    };

    // We "sort" it among his peers.
    loop {

        // Get the previous and next item ModelIndex on the list.
        let item_index_prev = item_index.sibling(item_index.row() - 1, 0);
        let item_index_next = item_index.sibling(item_index.row() + 1, 0);

        // Depending on the direction we have to move, get the second item's index.
        let item_sibling_index = if direction { item_index_prev } else { item_index_next };

        // If the sibling is valid...
        if item_sibling_index.is_valid() {

            // Get the Item sibling to our current Item.
            let item_sibling;
            unsafe { item_sibling = parent.as_mut().unwrap().child(item_sibling_index.row()); }

            // Get the path of the previous item.
            let path = get_path_from_item(model, item_sibling, true);

            // Send the Path to the Background Thread, and get the type of the item.
            sender_qt.send(Commands::GetTypeOfPath).unwrap();
            sender_qt_data.send(Data::VecString(path)).unwrap();
            let item_sibling_type = if let Data::TreePathType(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

            // If both are of the same type...
            if item_type == item_sibling_type {

                // Get both texts.
                let item_name: String;
                let sibling_name: String;
                unsafe { item_name = QString::to_std_string(&item.as_mut().unwrap().text()); }
                unsafe { sibling_name = QString::to_std_string(&item_sibling.as_mut().unwrap().text()); }

                // Depending on our direction, we sort one way or another
                if direction {

                    // For the previous item...
                    let name_list = vec![sibling_name.to_owned(), item_name.to_owned()];
                    let mut name_list_sorted = vec![sibling_name.to_owned(), item_name.to_owned()];
                    name_list_sorted.sort();

                    // If the order hasn't changed, we're done.
                    if name_list == name_list_sorted { break; }

                    // If they have changed positions...
                    else {

                        // Move the item one position above.
                        let item_x;
                        unsafe { item_x = parent.as_mut().unwrap().take_row(item_index.row()); }
                        unsafe { parent.as_mut().unwrap().insert_row(item_sibling_index.row(), &item_x); }
                        unsafe { item = parent.as_mut().unwrap().child(item_sibling_index.row()); }
                        unsafe { item_index = item.as_mut().unwrap().index(); }
                    }
                } else {

                    // For the next item...
                    let name_list = vec![item_name.to_owned(), sibling_name.to_owned()];
                    let mut name_list_sorted = vec![item_name.to_owned(), sibling_name.to_owned()];
                    name_list_sorted.sort();

                    // If the order hasn't changed, we're done.
                    if name_list == name_list_sorted { break; }

                    // If they have changed positions...
                    else {

                        // Move the item one position below.
                        let item_x;
                        unsafe { item_x = parent.as_mut().unwrap().take_row(item_index.row()); }
                        unsafe { parent.as_mut().unwrap().insert_row(item_sibling_index.row(), &item_x); }
                        unsafe { item = parent.as_mut().unwrap().child(item_sibling_index.row()); }
                        unsafe { item_index = item.as_mut().unwrap().index(); }
                    }
                }
            }

            // If the top one is a File and the bottom one a Folder, it's an special situation. Just swap them.
            else if item_type == TreePathType::Folder(vec![String::new()]) && item_sibling_type == TreePathType::File(vec![String::new()]) {

                // We swap them, and update them for the next loop.
                let item_x;
                unsafe { item_x = parent.as_mut().unwrap().take_row(item_index.row()); }
                unsafe { parent.as_mut().unwrap().insert_row(item_sibling_index.row(), &item_x); }
                unsafe { item = parent.as_mut().unwrap().child(item_sibling_index.row()); }
                unsafe { item_index = item.as_mut().unwrap().index(); }
            }

            // If the type is different and it's not an special situation, we can't move anymore.
            else { break; }
        }

        // If the Item is invalid, we can't move anymore.
        else { break; }
    }
}
