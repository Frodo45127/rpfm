// In this file are all the helper functions used by the UI (mainly Qt here)
extern crate chrono;
extern crate qt_widgets;
extern crate qt_gui;
extern crate qt_core;
extern crate cpp_utils;
extern crate serde_json;

use qt_widgets::action::Action;
use qt_widgets::combo_box::ComboBox;
use qt_widgets::dialog::Dialog;
use qt_widgets::grid_layout::GridLayout;
use qt_widgets::file_dialog::{FileDialog, FileMode};
use qt_widgets::label::Label;
use qt_widgets::layout::Layout;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::main_window::MainWindow;
use qt_widgets::message_box::{MessageBox, Icon};
use qt_widgets::push_button::PushButton;
use qt_widgets::tree_view::TreeView;
use qt_widgets::widget::Widget;

use qt_gui::brush::Brush;
use qt_gui::icon;
use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::connection::Signal;
use qt_core::event_loop::EventLoop;
use qt_core::flags::Flags;
use qt_core::item_selection::ItemSelection;
use qt_core::qt::GlobalColor;
use qt_core::slots::{SlotBool, SlotNoArgs, SlotModelIndexRef};
use qt_core::variant::Variant;
use cpp_utils::StaticCast;

use chrono::NaiveDateTime;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver};
use std::cmp::Ordering;
use std::path::PathBuf;
use std::fmt::Display;

use QString;
use AppUI;
use Commands;
use common::*;
use error::{Error, ErrorKind, Result};
use packedfile::*;
use packedfile::db::*;
use packedfile::db::schemas::*;
use packedfile::loc::*;

pub mod packedfile_db;
pub mod packedfile_loc;
pub mod packedfile_text;
pub mod packedfile_image;
pub mod packedfile_rigidmodel;
pub mod settings;
pub mod shortcuts;
pub mod updater;

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
        }
    }

    /// This function creates a new "Add From PackFile" struct and returns it.
    pub fn new_with_grid(
        rpfm_path: PathBuf,
        sender_qt: Sender<Commands>,
        sender_qt_data: &Sender<Result<Vec<u8>>>,
        receiver_qt: &Rc<RefCell<Receiver<Result<Vec<u8>>>>>,
        app_ui: AppUI,
        is_folder_tree_view_locked: &Rc<RefCell<bool>>,
        is_modified: &Rc<RefCell<bool>>,
        is_packedfile_opened: &Rc<RefCell<bool>>
    ) -> Self {

        // Create the stuff.
        let tree_view = TreeView::new().into_raw();
        let tree_model = StandardItemModel::new(()).into_raw();
        let exit_button = PushButton::new(&QString::from_std_str("Exit 'Add from Packfile' Mode")).into_raw();

        // Configure it.
        unsafe { tree_view.as_mut().unwrap().set_model(tree_model as *mut AbstractItemModel); }
        unsafe { tree_view.as_mut().unwrap().set_header_hidden(true); }
        unsafe { tree_view.as_mut().unwrap().set_expands_on_double_click(false); }

        // Add all the stuff to the Grid.
        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((exit_button as *mut Widget, 0, 0, 1, 1)); }
        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((tree_view as *mut Widget, 1, 0, 1, 1)); }

        // Create the slots for the stuff we need.
        let slots = Self {

            // This slot is used to copy something from one PackFile to the other when pressing the "<=" button.
            copy: SlotModelIndexRef::new(clone!(
                rpfm_path,
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
                    sender_qt_data.send(serde_json::to_vec(&item_path).map_err(From::from)).unwrap();

                    // Disable the Main Window (so we can't do other stuff).
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                    // Prepare the event loop, so we don't hang the UI while the background thread is working.
                    let mut event_loop = EventLoop::new();

                    // Until we receive a response from the worker thread...
                    loop {

                        // Get the response from the other thread.
                        let response: Result<(Vec<Vec<String>>)> = check_message_validity_tryrecv(&receiver_qt);

                        // Check what response we got.
                        match response {

                            // If we got a message...
                            Ok(paths) => {

                                // Update the TreeView.
                                update_treeview(
                                    &rpfm_path,
                                    &sender_qt,
                                    &sender_qt_data,
                                    receiver_qt.clone(),
                                    app_ui.window,
                                    app_ui.folder_tree_view,
                                    app_ui.folder_tree_model,
                                    TreeViewOperation::Add(paths),
                                );

                                // Set the mod as "Modified". This is an exception for the path, as it'll be painted later on.
                                *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                // Break the loop.
                                break;
                            }

                            // If we got an error...
                            Err(error) => {

                                // We must check what kind of error it's.
                                match error.kind() {

                                    // If it's "Message Empty", do nothing.
                                    ErrorKind::MessageSystemEmpty => {},

                                    // TODO: Depurate this into multiple errors.
                                    // If there is any other error, report it and break the loop.
                                    _ => {
                                        show_dialog(app_ui.window, true, error.kind());
                                        break;
                                    }
                                }
                            }
                        }

                        // Keep the UI responsive.
                        event_loop.process_events(());
                    }

                    // Re-enable the Main Window.
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                }
            )),

            // This slot is used to exit the "Add from PackFile" view, returning to the normal state of the program.
            exit: SlotNoArgs::new(clone!(
                sender_qt,
                is_packedfile_opened,
                is_folder_tree_view_locked => move || {

                    // Reset the Secondary PackFile.
                    sender_qt.send(Commands::ResetPackFileExtra).unwrap();

                    // Destroy the "Add from PackFile" stuff.
                    purge_them_all(&app_ui, &is_packedfile_opened);

                    // Show the "Tips".
                    display_help_tips(&app_ui);

                    // Unlock the TreeView so it can load PackedFiles again.
                    *is_folder_tree_view_locked.borrow_mut() = false;
                }
            )),
        };

        // Actions for the slots...
        unsafe { tree_view.as_ref().unwrap().signals().double_clicked().connect(&slots.copy); }
        unsafe { exit_button.as_ref().unwrap().signals().released().connect(&slots.exit); }

        // Update the new TreeView.
        update_treeview(
            &rpfm_path,
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

/// This function creates the entire "Rename" dialog. It returns the new name of the Item, or
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

    // Set it Modal, so you can't touch the Main Window with this dialog open.
    dialog.set_modal(true);

    // Resize the Dialog.
    dialog.resize((300, 0));

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

    // Resize the Dialog.
    dialog.resize((300, 0));

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
    receiver: &Rc<RefCell<Receiver<Result<Vec<u8>>>>>,
    packed_file_type: PackedFileType
) -> Option<Result<PackedFileType>> {

    //-------------------------------------------------------------------------------------------//
    // Creating the New PackedFile Dialog...
    //-------------------------------------------------------------------------------------------//

    // Create the "New PackedFile" Dialog.
    let mut dialog;
    unsafe { dialog = Dialog::new_unsafe(app_ui.window as *mut Widget); }

    // Change his title.
    match packed_file_type {
        PackedFileType::Loc(_) => dialog.set_window_title(&QString::from_std_str("New Loc PackedFile")),
        PackedFileType::DB(_,_,_) => dialog.set_window_title(&QString::from_std_str("New DB Table")),
        PackedFileType::Text(_) => dialog.set_window_title(&QString::from_std_str("New Text PackedFile")),
    }

    // Set it Modal, so you can't touch the Main Window with this dialog open.
    dialog.set_modal(true);

    // Resize the Dialog.
    dialog.resize((300, 0));

    // Create the main Grid.
    let main_grid = GridLayout::new().into_raw();

    // Create the "New Name" LineEdit.
    let mut new_packed_file_name_edit = LineEdit::new(());

    // Set the current name as default.
    new_packed_file_name_edit.set_text(&QString::from_std_str("new_file"));

    // Create the "Create" button.
    let create_button = PushButton::new(&QString::from_std_str("Create")).into_raw();

    // Create a dropdown to select the table.
    let mut table_dropdown = ComboBox::new();
    let mut table_model = StandardItemModel::new(());
    unsafe { table_dropdown.set_model(table_model.static_cast_mut()); }

    // Add all the widgets to the main grid.
    unsafe { main_grid.as_mut().unwrap().add_widget((new_packed_file_name_edit.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((create_button as *mut Widget, 0, 1, 1, 1)); }

    // And the Main Grid to the Dialog...
    unsafe { dialog.set_layout(main_grid as *mut Layout); }

    // If it's a DB Table...
    if let PackedFileType::DB(_,_,_) = packed_file_type {

        // Get the current schema.
        sender.send(Commands::GetSchema).unwrap();
        let schema: Option<Schema> = match check_message_validity_recv(&receiver) {
            Ok(data) => data,
            Err(_) => panic!(THREADS_MESSAGE_ERROR),
        };

        // Check if we actually have an schema.
        match schema {

            // If we have an schema...
            Some(schema) => {

                // Add every table to the dropdown.
                schema.tables_definitions.iter().for_each(|x| table_dropdown.add_item(&QString::from_std_str(&x.name)));

                // Add the dropdown to the dialog.
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

                // Get the current schema.
                sender.send(Commands::GetSchema).unwrap();
                let schema: Option<Schema> = match check_message_validity_recv(&receiver) {
                    Ok(data) => data,
                    Err(_) => panic!(THREADS_MESSAGE_ERROR),
                };

                // Check if we actually have an schema.
                match schema {

                    // If we have an schema...
                    Some(schema) => {

                        // Get the table and his version.
                        let table = table_dropdown.current_text().to_std_string();
                        let table_schema = schema.tables_definitions.iter().filter(|x| x.name == table).cloned().collect::<Vec<TableDefinitions>>();
                        let mut versions = table_schema[0].versions.iter().map(|x| x.version).collect::<Vec<u32>>();
                        versions.sort();
                        let version = versions[0];

                        Some(Ok(PackedFileType::DB(packed_file_name, table, version)))
                    }

                    // If we don't have an schema, return Some(Error).
                    None => return Some(Err(Error::from(ErrorKind::SchemaNotFound))),
                }
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

    // Create the "Mass-Import TSV" Dialog.
    let dialog;
    unsafe { dialog = Dialog::new_unsafe(app_ui.window as *mut Widget).into_raw(); }

    // Change his title.
    unsafe { dialog.as_mut().unwrap().set_window_title(&QString::from_std_str("Mass-Import TSV Files")); }

    // Set it Modal, so you can't touch the Main Window with this dialog open.
    unsafe { dialog.as_mut().unwrap().set_modal(true); }

    // Resize the Dialog.
    unsafe { dialog.as_mut().unwrap().resize((300, 0)); }

    // Create the main Grid.
    let main_grid = GridLayout::new().into_raw();

    // Create the "Files to import" Label.
    let files_to_import_label = Label::new(&QString::from_std_str("Files to import: 0.")).into_raw();

    // Create the "..." button.
    let select_files_button = PushButton::new(&QString::from_std_str("...")).into_raw();

    // Create the "Imported File's Name" LineEdit.
    let mut imported_files_name_line_edit = LineEdit::new(());

    // Set a dummy name as default.
    imported_files_name_line_edit.set_text(&QString::from_std_str("new_imported_file"));

    // Create the "Import" button.
    let import_button = PushButton::new(&QString::from_std_str("Import")).into_raw();

    // Add all the widgets to the main grid.
    unsafe { main_grid.as_mut().unwrap().add_widget((files_to_import_label as *mut Widget, 0, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((select_files_button as *mut Widget, 0, 1, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((imported_files_name_line_edit.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((import_button as *mut Widget, 1, 1, 1, 1)); }

    // And the Main Grid to the Dialog...
    unsafe { dialog.as_mut().unwrap().set_layout(main_grid as *mut Layout); }

    //-------------------------------------------------------------------------------------------//
    // Actions for the Mass-Import TSV Dialog...
    //-------------------------------------------------------------------------------------------//

    // Create the list of Paths to import.
    let files_to_import = Rc::new(RefCell::new(vec![]));

    // What happens when we hit the "..." button.
    let slot_select_files = SlotNoArgs::new(clone!(
        files_to_import => move || {

            // Create the FileDialog to get the PackFile to open.
            let mut file_dialog;
            unsafe { file_dialog = FileDialog::new_unsafe((
                dialog as *mut Widget,
                &QString::from_std_str("Select TSV Files to Import..."),
            )); }

            // Filter it so it only shows TSV Files.
            file_dialog.set_name_filter(&QString::from_std_str("TSV Files (*.tsv)"));

            // Set it to accept multiple files at once.
            file_dialog.set_file_mode(FileMode::ExistingFiles);

            // Run it and expect a response (1 => Accept, 0 => Cancel).
            if file_dialog.exec() == 1 {

                // Get the path of the selected files and turn it in a Rust's PathBuf.
                let selected_files = file_dialog.selected_files();
                for index in 0..selected_files.count(()) {
                    files_to_import.borrow_mut().push(PathBuf::from(file_dialog.selected_files().at(index).to_std_string()));
                }

                // Update the label with the amount of files to import.
                unsafe { files_to_import_label.as_mut().unwrap().set_text(&QString::from_std_str(&format!("Files to import: {}.", selected_files.count(())))); }
            }
        }
    ));

    // What happens when we hit the "..." button.
    unsafe { select_files_button.as_mut().unwrap().signals().released().connect(&slot_select_files); }

    // What happens when we hit the "Import" button.
    unsafe { import_button.as_mut().unwrap().signals().released().connect(&dialog.as_mut().unwrap().slots().accept()); }

    unsafe {
        // Show the Dialog and, if we hit the "Create" button...
        if dialog.as_mut().unwrap().exec() == 1 {

            // Get the text from the LineEdit.
            let packed_file_name = imported_files_name_line_edit.text().to_std_string();

            // Return the name of the files and the list of paths.
            Some((packed_file_name, files_to_import.borrow().to_vec()))
        }

        // In any other case, we return None.
        else { None }
    }
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

//----------------------------------------------------------------------------//
//                    Enums & Structs needed for the UI
//----------------------------------------------------------------------------//

/// Enum `TreeViewOperation`: This enum has the different possible operations we want to do over a TreeView. The options are:
/// - `Build`: Build the entire TreeView from nothing. Requires a bool, depending if the PackFile is editable or not.
/// - `Add`: Add a File/Folder to the TreeView. Requires the path in the TreeView, without the mod's name.
/// - `DeleteSelected`: Removes whatever is selected from the TreeView. It requires the TreePathType of whatever you want to delete.
/// - `DeleteUnselected`: Remove the File/Folder corresponding to the TreePathType we provide from the TreeView. It requires the TreePathType of whatever you want to delete.
/// - `Rename`: Change the name of a File/Folder from the TreeView. Requires the TreePathType of whatever you want to rename and the new name.
#[derive(Clone, Debug)]
pub enum TreeViewOperation {
    Build(bool),
    Add(Vec<Vec<String>>),
    DeleteSelected(TreePathType),
    DeleteUnselected(TreePathType),
    Rename(TreePathType, String),
}

/// Enum `ItemVisualStatus`: This enum represents the status of modification of an item in a TreeView.
#[derive(PartialEq)]
pub enum ItemVisualStatus {
    Added,
    Modified,
    AddedModified,
    Untouched,
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
struct Icons {
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

/// Implementation of "Icons".
impl Icons {

    /// This function creates a list of Icons from certain paths in disk.
    fn new(rpfm_path: &PathBuf) -> Self {

        // Get the Path as a String, so Qt can understand it.
        let rpfm_path_string = rpfm_path.to_string_lossy().as_ref().to_string();

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

/// This function deletes whatever it's in the right side of the screen, leaving it empty.
/// Also, each time this triggers we consider there is no PackedFile open.
pub fn purge_them_all(app_ui: &AppUI, is_packedfile_opened: &Rc<RefCell<bool>>) {

    // Black magic.
    unsafe {
        for _ in 0..app_ui.packed_file_layout.as_mut().unwrap().count() {
            let child = app_ui.packed_file_layout.as_mut().unwrap().take_at(0);
            child.as_mut().unwrap().widget().as_mut().unwrap().close();
            app_ui.packed_file_layout.as_mut().unwrap().remove_item(child);
        }
    }

    // Set it as not having an opened PackedFile, just in case.
    *is_packedfile_opened.borrow_mut() = false;

    // Just in case what was open before this was a DB Table, make sure the "Game Selected" menu is re-enabled.
    unsafe { app_ui.game_selected_group.as_mut().unwrap().set_enabled(true); }

    // Fix the column and row stretch caused by the DB Decoder.
    unsafe { app_ui.packed_file_layout.as_mut().unwrap().set_column_stretch(1, 0); }
    unsafe { app_ui.packed_file_layout.as_mut().unwrap().set_row_stretch(0, 0); }
    unsafe { app_ui.packed_file_layout.as_mut().unwrap().set_row_stretch(2, 0); }
}

/// This function shows the tips in the PackedFile View. Remember to call "purge_them_all" before this!
pub fn display_help_tips(app_ui: &AppUI) {

    let label = Label::new(&QString::from_std_str("Welcome to Rusted PackFile Manager! Here you have some tips on how to use it:
    - If you just downloaded, go to 'PackFile/Preferences' and configure there what you need.
    - Then, open a PackFile of the Games you have, go to 'Special Stuff/YourGames/Generate Dependency PackFile'. Once per game.
    - Make sure the right game is selected under 'Game Selected' before you do it.
    - Once you've done that for each game, RPFM will be ready to be used.
    - Remember to re-generate the 'Dependency PackFiles' once their game is updated.
    - To know what each option in 'Preferences' do, left the mouse over the option for one second and a tooltip will pop up.
    - In the 'About' Menu, in 'About RPFM' you can find links to the Source Code and the Patreon of the Project. Both places are suitable to provide feedback.")).into_raw();

    unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((label as *mut Widget, 0, 0, 1, 1)); }
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
    status: ItemVisualStatus
) {

    // Get the color we need to apply.
    let color = match &status {
        ItemVisualStatus::Added => GlobalColor::Green,
        ItemVisualStatus::Modified => GlobalColor::Yellow,
        ItemVisualStatus::AddedModified => GlobalColor::Magenta,
        ItemVisualStatus::Untouched => GlobalColor::Transparent,
    };

    // Get the full path of the item.
    let full_path = get_path_from_item(model, item, true);

    // Get the times we must to go up until we reach the parent.
    let cycles = if full_path.len() > 0 { full_path.len() - 1 } else { 0 };

    // Paint it like one of your french girls.
    unsafe { item.as_mut().unwrap().set_background(&Brush::new(color.clone())); }

    // Get his parent.
    let mut parent;
    unsafe { parent = item.as_mut().unwrap().parent(); }

    // Loop through his parents until we reach the PackFile
    for _ in 0..cycles {

        // Get the color of the Parent.
        let parent_color;
        unsafe { parent_color = parent.as_mut().unwrap().background().color().name(()).to_std_string(); }

        // Get the status of the Parent depending on his color.
        let parent_status = match &*parent_color {
            "#00ff00" => ItemVisualStatus::Added,
            "#ffff00" => ItemVisualStatus::Modified,
            "#ff00ff" => ItemVisualStatus::AddedModified,
            "#000000" | _ => ItemVisualStatus::Untouched,
        };

        // Paint it depending on his status.
        match parent_status {

            // If it's Added...
            ItemVisualStatus::Added => {

                // If the new status is "Modified", turn it into "AddedModified"
                if status == ItemVisualStatus::Modified { unsafe { parent.as_mut().unwrap().set_background(&Brush::new(GlobalColor::Magenta)); } }
            },

            // If it's Modified...
            ItemVisualStatus::Modified => {

                // If the new status is "Added", turn it into "AddedModified"
                if status == ItemVisualStatus::Added { unsafe { parent.as_mut().unwrap().set_background(&Brush::new(GlobalColor::Magenta)); } }
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
    icons: &Icons,
    icon_type: IconType,
) {

    // Depending on the IconType we receive...
    match icon_type {

        // For PackFiles.
        IconType::PackFile(editable) => {
            if editable { unsafe { item.as_mut().unwrap().set_icon(&icons.packfile_editable); } }
            else { unsafe { item.as_mut().unwrap().set_icon(&icons.packfile_locked); } }
        },

        // For folders.
        IconType::Folder => unsafe { item.as_mut().unwrap().set_icon(&icons.folder); },

        // For files.
        IconType::File(path) => {

            // Get the name of the file.
            let packed_file_name = path.last().unwrap();

            // If it's in the "db" folder, it's a DB PackedFile (or you put something were it shouldn't be).
            if path[0] == "db" { unsafe { item.as_mut().unwrap().set_icon(&icons.table); } }

            // If it ends in ".loc", it's a localisation PackedFile.
            else if packed_file_name.ends_with(".loc") { unsafe { item.as_mut().unwrap().set_icon(&icons.table); } }

            // If it ends in ".rigid_model_v2", it's a RigidModel PackedFile.
            else if packed_file_name.ends_with(".rigid_model_v2") { unsafe { item.as_mut().unwrap().set_icon(&icons.rigid_model); } }

            // If it ends in any of these, it's a plain text PackedFile.
            else if packed_file_name.ends_with(".lua") { unsafe { item.as_mut().unwrap().set_icon(&icons.text_generic); } }
            else if packed_file_name.ends_with(".xml") { unsafe { item.as_mut().unwrap().set_icon(&icons.text_xml); } }
            else if packed_file_name.ends_with(".xml.shader") { unsafe { item.as_mut().unwrap().set_icon(&icons.text_xml); } }
            else if packed_file_name.ends_with(".xml.material") { unsafe { item.as_mut().unwrap().set_icon(&icons.text_xml); } }
            else if packed_file_name.ends_with(".variantmeshdefinition") { unsafe { item.as_mut().unwrap().set_icon(&icons.text_xml); } }
            else if packed_file_name.ends_with(".environment") { unsafe { item.as_mut().unwrap().set_icon(&icons.text_xml); } }
            else if packed_file_name.ends_with(".lighting") { unsafe { item.as_mut().unwrap().set_icon(&icons.text_generic); } }
            else if packed_file_name.ends_with(".wsmodel") { unsafe { item.as_mut().unwrap().set_icon(&icons.text_generic); } }
            else if packed_file_name.ends_with(".csv") { unsafe { item.as_mut().unwrap().set_icon(&icons.text_csv); } }
            else if packed_file_name.ends_with(".tsv") { unsafe { item.as_mut().unwrap().set_icon(&icons.text_csv); } }
            else if packed_file_name.ends_with(".inl") { unsafe { item.as_mut().unwrap().set_icon(&icons.text_generic); } }
            else if packed_file_name.ends_with(".battle_speech_camera") { unsafe { item.as_mut().unwrap().set_icon(&icons.text_generic); } }
            else if packed_file_name.ends_with(".bob") { unsafe { item.as_mut().unwrap().set_icon(&icons.text_generic); } }
            else if packed_file_name.ends_with(".cindyscene") { unsafe { item.as_mut().unwrap().set_icon(&icons.text_generic); } }
            else if packed_file_name.ends_with(".cindyscenemanager") { unsafe { item.as_mut().unwrap().set_icon(&icons.text_generic); } }
            //else if packed_file_name.ends_with(".benchmark") || // This one needs special decoding/encoding.
            else if packed_file_name.ends_with(".txt") { unsafe { item.as_mut().unwrap().set_icon(&icons.text_txt); } }

            // If it ends in any of these, it's an image.
            else if packed_file_name.ends_with(".jpg") { unsafe { item.as_mut().unwrap().set_icon(&icons.image_jpg); } }
            else if packed_file_name.ends_with(".jpeg") { unsafe { item.as_mut().unwrap().set_icon(&icons.image_jpg); } }
            else if packed_file_name.ends_with(".tga") { unsafe { item.as_mut().unwrap().set_icon(&icons.image_generic); } }
            else if packed_file_name.ends_with(".dds") { unsafe { item.as_mut().unwrap().set_icon(&icons.image_generic); } }
            else if packed_file_name.ends_with(".png") { unsafe { item.as_mut().unwrap().set_icon(&icons.image_png); } }

            // Otherwise, it's a generic file.
            else { unsafe { item.as_mut().unwrap().set_icon(&icons.file); } }
        }
    }
}

/// This function takes care of EVERY operation that manipulates the provided TreeView.
/// It does one thing or another, depending on the operation we provide it.
pub fn update_treeview(
    rpfm_path: &PathBuf,
    sender_qt: &Sender<Commands>,
    sender_qt_data: &Sender<Result<Vec<u8>>>,
    receiver_qt: Rc<RefCell<Receiver<Result<Vec<u8>>>>>,
    window: *mut MainWindow,
    tree_view: *mut TreeView,
    model: *mut StandardItemModel,
    operation: TreeViewOperation,
) {

    // Get the Icons for the TreeView.
    // TODO: Move this to const fn when it reaches stable in rust.
    let icons = Icons::new(&rpfm_path);

    // We act depending on the operation requested.
    match operation {

        // If we want to build a new TreeView...
        TreeViewOperation::Build(is_extra_packfile) => {

            // Depending on the PackFile we want to build the TreeView with, we ask for his data.
            if is_extra_packfile { sender_qt.send(Commands::GetPackFileExtraDataForTreeView).unwrap(); }
            else { sender_qt.send(Commands::GetPackFileDataForTreeView).unwrap(); }

            let pack_file_data: (String, u32, Vec<Vec<String>>)  = match check_message_validity_recv(&receiver_qt) {
                Ok(data) => data,
                Err(_) => panic!(THREADS_MESSAGE_ERROR)
            };

            // First, we clean the TreeStore and whatever was created in the TreeView.
            unsafe { model.as_mut().unwrap().clear(); }

            // Second, we set as the big_parent, the base for the folders of the TreeView, a fake folder
            // with the name of the PackFile. All big things start with a lie.
            let mut big_parent = StandardItem::new(&QString::from_std_str(pack_file_data.0)).into_raw();

            // Get his last modified date and show it in a tooltip.
            unsafe { big_parent.as_mut().unwrap().set_tool_tip(&QString::from_std_str(format!("Last Modified: {:?}", NaiveDateTime::from_timestamp(i64::from(pack_file_data.1), 0)))); }

            // Also, set it as not editable by the user. Otherwise will cause problems when renaming.
            unsafe { big_parent.as_mut().unwrap().set_editable(false); }

            // Add the Big Parent to the Model.
            unsafe { model.as_mut().unwrap().append_row_unsafe(big_parent); }

            // Give it an Icon.
            set_icon_to_item(big_parent, &icons, IconType::PackFile(is_extra_packfile));

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
                        set_icon_to_item(file, &icons, IconType::File(path));
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
                                    set_icon_to_item(folder, &icons, IconType::Folder);

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
                                set_icon_to_item(folder, &icons, IconType::Folder);

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
                            sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();
                            let item_type: TreePathType = match check_message_validity_recv(&receiver_qt) {
                                Ok(data) => data,
                                Err(_) => panic!(THREADS_MESSAGE_ERROR)
                            };

                            // Act depending on the Type of the Path.
                            match item_type {

                                // If it's a folder, give it an Icon.
                                TreePathType::Folder(_) => set_icon_to_item(item, &icons, IconType::Folder),

                                // If it's a folder, give it an Icon.
                                TreePathType::File((ref path,_)) => set_icon_to_item(item, &icons, IconType::File(path.to_vec())),

                                // Any other type, ignore it.
                                _ => {},
                            }

                            // Paint it like that parrot you painted yesterday.
                            paint_treeview(item, model, ItemVisualStatus::Added);

                            // Sort the TreeView.
                            sort_item_in_tree_view(
                                sender_qt,
                                sender_qt_data,
                                receiver_qt.clone(),
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
                                    set_icon_to_item(folder, &icons, IconType::Folder);

                                    // This is our parent now.
                                    let index = parent.as_ref().unwrap().row_count() - 1;
                                    parent = parent.as_mut().unwrap().child(index);

                                    // Sort the TreeView.
                                    sort_item_in_tree_view(
                                        sender_qt,
                                        sender_qt_data,
                                        receiver_qt.clone(),
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
                                set_icon_to_item(folder, &icons, IconType::Folder);

                                // This is our parent now.
                                let index = parent.as_ref().unwrap().row_count() - 1;
                                parent = parent.as_mut().unwrap().child(index);

                                // Sort the TreeView.
                                sort_item_in_tree_view(
                                    sender_qt,
                                    sender_qt_data,
                                    receiver_qt.clone(),
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
                        &rpfm_path,
                        &sender_qt,
                        &sender_qt_data,
                        receiver_qt.clone(),
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
                TreePathType::File((path,_)) => {

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
                TreePathType::Folder(_) | TreePathType::File((_,_)) => {

                    // Get the item.
                    let item;
                    unsafe { item = model.as_mut().unwrap().item_from_index(selection); }

                    // Paint it as "modified".
                    paint_treeview(item, model, ItemVisualStatus::Modified);

                    // Sort it.
                    sort_item_in_tree_view(
                        sender_qt,
                        sender_qt_data,
                        receiver_qt.clone(),
                        model,
                        item,
                        path_type
                    );
                }

                // In any other case, don't do anything.
                _ => {},
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
    sender_qt_data: &Sender<Result<Vec<u8>>>,
    receiver_qt: Rc<RefCell<Receiver<Result<Vec<u8>>>>>,
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
        sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();
        match check_message_validity_recv(&receiver_qt) {
            Ok(data) => data,
            Err(_) => panic!(THREADS_MESSAGE_ERROR)
        }
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
        sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();
        match check_message_validity_recv(&receiver_qt) {
            Ok(data) => data,
            Err(_) => panic!(THREADS_MESSAGE_ERROR)
        }
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
    else if item_type_prev == TreePathType::Folder(vec![String::new()]) && item_type_next == TreePathType::File((vec![String::new()], 1)) {
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
            sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();
            let item_sibling_type: TreePathType = match check_message_validity_recv(&receiver_qt) {
                Ok(data) => data,
                Err(_) => panic!(THREADS_MESSAGE_ERROR)
            };

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
            else if item_type == TreePathType::Folder(vec![String::new()]) && item_sibling_type == TreePathType::File((vec![String::new()], 1)) {

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
