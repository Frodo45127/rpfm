//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this file are all the helper functions used by the UI (mainly Qt here)
use qt_widgets::action::Action;
use qt_widgets::check_box::CheckBox;
use qt_widgets::combo_box::ComboBox;
use qt_widgets::dialog::Dialog;
use qt_widgets::file_dialog::{FileDialog, FileMode};
use qt_widgets::grid_layout::GridLayout;
use qt_widgets::group_box::GroupBox;
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
use qt_gui::key_sequence::KeySequence;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::connection::Signal;
use qt_core::flags::Flags;
use qt_core::model_index::ModelIndex;
use qt_core::object::Object;
use qt_core::qt::ShortcutContext;
use qt_core::reg_exp::RegExp;
use qt_core::slots::{SlotBool, SlotNoArgs, SlotStringRef, SlotModelIndexRef};
use qt_core::sort_filter_proxy_model::SortFilterProxyModel;

use cpp_utils::{CppBox, StaticCast};

use chrono::NaiveDateTime;
use std::collections::BTreeMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver};
use std::cmp::Ordering;
use std::path::PathBuf;
use std::{fmt, fmt::Display, fmt::Debug};
use std::f32;

use crate::RPFM_PATH;
use crate::SHORTCUTS;
use crate::SETTINGS;
use crate::SCHEMA;
use crate::IS_MODIFIED;
use crate::IS_FOLDER_TREE_VIEW_LOCKED;
use crate::ORANGE;
use crate::SLIGHTLY_DARKER_GREY;
use crate::MEDIUM_DARKER_GREY;
use crate::DARK_GREY;
use crate::KINDA_WHITY_GREY;
use crate::EVEN_MORE_WHITY_GREY;
use crate::QString;
use crate::AppUI;
use crate::Commands;
use crate::Data;
use crate::common::*;
use crate::common::communications::*;
use crate::error::{Error, ErrorKind, Result};
use crate::packedfile::*;
use crate::packedfile::db::*;
use crate::packedfile::db::schemas::*;
use crate::ui::packfile_treeview::*;
use crate::ui::table_state::TableStateData;

pub mod packedfile_table;
pub mod packedfile_text;
pub mod packedfile_image;
pub mod packedfile_rigidmodel;
pub mod packfile_treeview;
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
        sender_qt: &Sender<Commands>,
        sender_qt_data: &Sender<Data>,
        receiver_qt: &Rc<RefCell<Receiver<Data>>>,
        app_ui: AppUI,
        packedfiles_open_in_packedfile_view: &Rc<RefCell<BTreeMap<i32, Rc<RefCell<Vec<String>>>>>>,
        global_search_explicit_paths: &Rc<RefCell<Vec<Vec<String>>>>,
        update_global_search_stuff: *mut Action,
        table_state_data: &Rc<RefCell<BTreeMap<Vec<String>, TableStateData>>>
    ) -> Self {

        // Create the widget that'll act as a container for the view.
        let widget = Widget::new().into_raw();
        let widget_layout = create_grid_layout_unsafe(widget);
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
                sender_qt,
                table_state_data,
                sender_qt_data,
                receiver_qt => move |_| {

                    // Get the file to get from the Right TreeView.
                    let selection_file_to_move = unsafe { tree_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection() };
                    if selection_file_to_move.count(()) == 1 {
                        let item_type = From::from(&get_item_types_from_selection(tree_view, None, tree_model)[0]);

                        // Ask the Background Thread to move the files, and send him the path.
                        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                        sender_qt.send(Commands::AddPackedFileFromPackFile).unwrap();
                        sender_qt_data.send(Data::PathType(item_type)).unwrap();

                        // Check what response we got.
                        match check_message_validity_tryrecv(&receiver_qt) {
                        
                            // If it's success....
                            Data::VecPathType(paths) => {

                                // Update the TreeView.
                                let paths = paths.iter().map(|x| From::from(x)).collect::<Vec<TreePathType>>();
                                update_treeview(
                                    &sender_qt,
                                    &sender_qt_data,
                                    &receiver_qt,
                                    &app_ui,
                                    app_ui.folder_tree_view,
                                    Some(app_ui.folder_tree_filter),
                                    app_ui.folder_tree_model,
                                    TreeViewOperation::Add(paths.to_vec()),
                                );

                                // Update the global search stuff, if needed.
                                let paths = paths.iter().map(|x| 
                                    match x {
                                        TreePathType::File(ref path) => path.to_vec(),
                                        TreePathType::Folder(ref path) => path.to_vec(),
                                        TreePathType::PackFile => vec![],
                                        TreePathType::None => unimplemented!(),
                                    }
                                ).collect::<Vec<Vec<String>>>();
                                global_search_explicit_paths.borrow_mut().append(&mut paths.to_vec());
                                unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }

                                // For each file added, remove it from the data history if exists.
                                for path in &paths {
                                    if table_state_data.borrow().get(path).is_some() {
                                        table_state_data.borrow_mut().remove(path);
                                    }
                                
                                    // Set it to not remove his color.
                                    let data = TableStateData::new_empty();
                                    table_state_data.borrow_mut().insert(path.to_vec(), data);
                                }
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
                }
            )),

            // This slot is used to exit the "Add from PackFile" view, returning to the normal state of the program.
            exit: SlotNoArgs::new(clone!(
                sender_qt,
                packedfiles_open_in_packedfile_view => move || {

                    // Reset the Secondary PackFile.
                    sender_qt.send(Commands::ResetPackFileExtra).unwrap();

                    // Destroy the "Add from PackFile" stuff.
                    purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);

                    // Show the "Tips".
                    display_help_tips(&app_ui);
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

        unsafe { tree_view_expand_all.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["expand_all"]))); }
        unsafe { tree_view_collapse_all.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["collapse_all"]))); }

        unsafe { tree_view_expand_all.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { tree_view_collapse_all.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }

        unsafe { tree_view.as_mut().unwrap().add_action(tree_view_expand_all); }
        unsafe { tree_view.as_mut().unwrap().add_action(tree_view_collapse_all); }

        // Update the new TreeView.
        update_treeview(
            &sender_qt,
            &sender_qt_data,
            &receiver_qt,
            &app_ui,
            tree_view,
            None,
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
pub fn create_rename_dialog(app_ui: &AppUI, selected_items: &[TreePathType]) -> Option<String> {

    // Create and configure the dialog.
    let mut dialog = unsafe { Dialog::new_unsafe(app_ui.window as *mut Widget) };
    dialog.set_window_title(&QString::from_std_str("Rename Selection"));
    dialog.set_modal(true);
    dialog.resize((400, 50));
    let main_grid = create_grid_layout_unsafe(dialog.static_cast_mut() as *mut Widget);

    // Create a little frame with some instructions.
    let instructions_frame = GroupBox::new(&QString::from_std_str("Instructions")).into_raw();
    let instructions_grid = create_grid_layout_unsafe(instructions_frame as *mut Widget);
    let mut instructions_label = Label::new(&QString::from_std_str(
    "\
It's easy, but you'll not understand it without an example, so here it's one:
 - Your files/folders says 'you' and 'I'.
 - Write 'whatever {x} want' in the box below.
 - Hit 'Accept'.
 - RPFM will turn that into 'whatever you want' and 'whatever I want' and call your files/folders that.
And, in case you ask, works with numeric cells too, as long as the resulting text is a valid number.
    "    
    ));
    unsafe { instructions_grid.as_mut().unwrap().add_widget((instructions_label.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }

    let mut rewrite_sequence_line_edit = LineEdit::new(());
    rewrite_sequence_line_edit.set_placeholder_text(&QString::from_std_str("Write here whatever you want. {x} it's your current name."));
    
    // If we only have one selected item, put his name by default in the rename dialog.
    if selected_items.len() == 1 { 
        if let TreePathType::File(path) | TreePathType::Folder(path) = &selected_items[0] {
            rewrite_sequence_line_edit.set_text(&QString::from_std_str(path.last().unwrap()));
        }
    }
    let accept_button = PushButton::new(&QString::from_std_str("Accept")).into_raw();

    unsafe { main_grid.as_mut().unwrap().add_widget((instructions_frame as *mut Widget, 0, 0, 1, 2)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((rewrite_sequence_line_edit.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((accept_button as *mut Widget, 1, 1, 1, 1)); }

    unsafe { accept_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

    if dialog.exec() == 1 { 
        let new_text = rewrite_sequence_line_edit.text().to_std_string();
        if new_text.is_empty() { None } else { Some(rewrite_sequence_line_edit.text().to_std_string()) } 
    } else { None }
}

/// This function creates the entire "New Folder" dialog. It returns the new name of the Folder, or
/// None if the dialog is canceled or closed.
pub fn create_new_folder_dialog(app_ui: &AppUI) -> Option<String> {

    //-------------------------------------------------------------------------------------------//
    // Creating the New Folder Dialog...
    //-------------------------------------------------------------------------------------------//

    // Create the "New Folder" Dialog and configure it.
    let mut dialog = unsafe { Dialog::new_unsafe(app_ui.window as *mut Widget) };
    dialog.set_window_title(&QString::from_std_str("New Folder"));
    dialog.set_modal(true);

    // Create the main Grid.
    let main_grid = create_grid_layout_unsafe(dialog.static_cast_mut() as *mut Widget);

    // Create the "New Folder" LineEdit and configure it.
    let mut new_folder_line_edit = LineEdit::new(());
    new_folder_line_edit.set_text(&QString::from_std_str("new_folder"));
    let new_folder_button = PushButton::new(&QString::from_std_str("New Folder")).into_raw();

    // Add all the widgets to the main grid.
    unsafe { main_grid.as_mut().unwrap().add_widget((new_folder_line_edit.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((new_folder_button as *mut Widget, 0, 1, 1, 1)); }

    //-------------------------------------------------------------------------------------------//
    // Actions for the New Folder Dialog...
    //-------------------------------------------------------------------------------------------//

    // What happens when we hit the "New Folder" button.
    unsafe { new_folder_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

    // Show the Dialog and, if we hit the "New Folder" button, return the new name.
    if dialog.exec() == 1 { Some(new_folder_line_edit.text().to_std_string()) }

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
    packed_file_type: &PackedFileType
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
    let main_grid = create_grid_layout_unsafe(dialog.static_cast_mut() as *mut Widget);
    let mut new_packed_file_name_edit = LineEdit::new(());
    let table_filter_line_edit = LineEdit::new(()).into_raw();
    let create_button = PushButton::new(&QString::from_std_str("Create")).into_raw();
    let mut table_dropdown = ComboBox::new();
    let table_filter = SortFilterProxyModel::new().into_raw();
    let mut table_model = StandardItemModel::new(());

    new_packed_file_name_edit.set_text(&QString::from_std_str("new_file"));
    unsafe { table_dropdown.set_model(table_model.static_cast_mut()); }
    unsafe { table_filter_line_edit.as_mut().unwrap().set_placeholder_text(&QString::from_std_str("Type here to filter the tables of the list. Works with Regex too!")); }

    // Add all the widgets to the main grid.
    unsafe { main_grid.as_mut().unwrap().add_widget((new_packed_file_name_edit.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((create_button as *mut Widget, 0, 1, 1, 1)); }

    // If it's a DB Table...
    if let PackedFileType::DB(_,_,_) = packed_file_type {

        // Get a list of all the tables currently in use by the selected game.
        sender.send(Commands::GetTableListFromDependencyPackFile).unwrap();
        let tables = if let Data::VecString(data) = check_message_validity_recv2(&receiver) { data } else { panic!(THREADS_MESSAGE_ERROR); };

        // Check if we actually have an schema.
        match *SCHEMA.lock().unwrap() {
            Some(ref schema) => {

                // Add every table to the dropdown if exists in the dependency database.
                schema.tables_definitions.iter().filter(|x| tables.contains(&x.name)).for_each(|x| table_dropdown.add_item(&QString::from_std_str(&x.name)));
                unsafe { table_filter.as_mut().unwrap().set_source_model(table_model.static_cast_mut()); }
                unsafe { table_dropdown.set_model(table_filter as *mut AbstractItemModel); }

                unsafe { main_grid.as_mut().unwrap().add_widget((table_dropdown.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
                unsafe { main_grid.as_mut().unwrap().add_widget((table_filter_line_edit as *mut Widget, 2, 0, 1, 1)); }
            }

            // If we don't have an schema, return Some(Error).
            None => return Some(Err(Error::from(ErrorKind::SchemaNotFound))),
        }
    }

    //-------------------------------------------------------------------------------------------//
    // Actions for the New PackedFile Dialog...
    //-------------------------------------------------------------------------------------------//

    // What happens when we search in the filter.
    let slot_table_filter_change_text = SlotStringRef::new(move |_| {
        let pattern = unsafe { RegExp::new(&table_filter_line_edit.as_mut().unwrap().text()) };
        unsafe { table_filter.as_mut().unwrap().set_filter_reg_exp(&pattern); }
    });

    // What happens when we hit the "Create" button.
    unsafe { create_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

    // What happens when we edit the search filter.
    unsafe { table_filter_line_edit.as_mut().unwrap().signals().text_changed().connect(&slot_table_filter_change_text); }

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
                    Data::I32(data) => data,
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
    let main_grid = create_grid_layout_unsafe(dialog as *mut Widget);
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

/// This function creates the entire "Global Search" dialog. It returns the search info (pattern, case_sensitive).
pub fn create_global_search_dialog(app_ui: &AppUI) -> Option<String> {

    let mut dialog  = unsafe { Dialog::new_unsafe(app_ui.window as *mut Widget) };
    dialog.set_window_title(&QString::from_std_str("Global Search"));
    dialog.set_modal(true);

    // Create the main Grid.
    let main_grid = create_grid_layout_unsafe(dialog.static_cast_mut() as *mut Widget);
    let mut pattern = LineEdit::new(());
    pattern.set_placeholder_text(&QString::from_std_str("Write here what you want to search."));

    let search_button = PushButton::new(&QString::from_std_str("Search")).into_raw();
    unsafe { main_grid.as_mut().unwrap().add_widget((pattern.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((search_button as *mut Widget, 0, 1, 1, 1)); }

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

/// This function creates the entire "Merge Tables" dialog. It returns the stuff set in it.
pub fn create_merge_tables_dialog(app_ui: &AppUI) -> Option<(String, bool)> {

    let mut dialog = unsafe { Dialog::new_unsafe(app_ui.window as *mut Widget) };
    dialog.set_window_title(&QString::from_std_str("Merge Tables"));
    dialog.set_modal(true);

    // Create the main Grid.
    let main_grid = create_grid_layout_unsafe(dialog.static_cast_mut() as *mut Widget);
    let mut name = LineEdit::new(());
    name.set_placeholder_text(&QString::from_std_str("Write the name of the new file here."));

    let mut delete_source_tables = CheckBox::new(&QString::from_std_str("Delete original tables"));

    let accept_button = PushButton::new(&QString::from_std_str("Accept")).into_raw();
    unsafe { main_grid.as_mut().unwrap().add_widget((name.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((delete_source_tables.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((accept_button as *mut Widget, 2, 0, 1, 1)); }

    // What happens when we hit the "Search" button.
    unsafe { accept_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

    // Execute the dialog.
    if dialog.exec() == 1 { 
        let text = name.text().to_std_string();
        let delete_source_tables = delete_source_tables.is_checked();
        if !text.is_empty() { Some((text, delete_source_tables)) }
        else { None }
    }
    
    // Otherwise, return None.
    else { None }
}

//----------------------------------------------------------------------------//
//                    Enums & Structs needed for the UI
//----------------------------------------------------------------------------//

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
    let mut dialog = unsafe { MessageBox::new_unsafe((
        icon,
        &QString::from_std_str(title),
        &QString::from_std_str(&text.to_string()),
        Flags::from_int(1024), // Ok button.
        window as *mut Widget,
    )) };

    // Run the dialog.
    dialog.exec();
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

    // Unlock the TreeView, in case it was locked.
    *IS_FOLDER_TREE_VIEW_LOCKED.lock().unwrap() = false;
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
    let widget_layout = create_grid_layout_unsafe(widget);
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
    is_delete_my_mod: bool
) -> bool {

    // If the mod has been modified...
    if *IS_MODIFIED.lock().unwrap() {

        // Create the dialog.
        let mut dialog = unsafe { MessageBox::new_unsafe((
            &QString::from_std_str("Rusted PackFile Manager"),
            &QString::from_std_str("<p>There are some changes yet to be saved.</p><p>Are you sure?</p>"),
            Icon::Warning,
            65536, // No
            16384, // Yes
            1, // By default, select yes.
            app_ui.window as *mut Widget,
        )) };

        // Run the dialog and get the response. Yes => 3, No => 4.
        if dialog.exec() == 3 { true } else { false }
    }

    // If we are going to delete a MyMod...
    else if is_delete_my_mod {

        // Create the dialog.
        let mut dialog = unsafe { MessageBox::new_unsafe((
            &QString::from_std_str("Rusted PackFile Manager"),
            &QString::from_std_str("<p>You are about to delete this <i>'MyMod'</i> from your disk.</p><p>There is no way to recover it after that.</p><p>Are you sure?</p>"),
            Icon::Warning,
            65536, // No
            16384, // Yes
            1, // By default, select yes.)
            app_ui.window as *mut Widget,
        )) };

        // Run the dialog and get the response. Yes => 3, No => 4.
        if dialog.exec() == 3 { true } else { false }
    }

    // Otherwise, we allow the change directly.
    else { true }
}

/// This function creates a GridLayout for the provided widget with the settings we want.
///
/// This is the safe version for CppBox.
pub fn create_grid_layout_safe(widget: &mut CppBox<Widget>) -> CppBox<GridLayout> {
    let mut widget_layout = GridLayout::new();
    unsafe { widget.set_layout(widget_layout.static_cast_mut() as *mut Layout); }
    
    // Due to how Qt works, if we want a decent look on windows, we have to do some specific tweaks there.
    if cfg!(target_os = "windows") {
        widget_layout.set_contents_margins((2, 2, 2, 2));
        widget_layout.set_spacing(1);
    }
    else {
        widget_layout.set_contents_margins((0, 0, 0, 0));
        widget_layout.set_spacing(0);            
    }

    widget_layout
}

/// This function creates a GridLayout for the provided widget with the settings we want.
///
/// This is the unsafe version for Pointers.
pub fn create_grid_layout_unsafe(widget: *mut Widget) -> *mut GridLayout {
    let widget_layout = GridLayout::new().into_raw();
    unsafe { widget.as_mut().unwrap().set_layout(widget_layout as *mut Layout); }
    
    // Due to how Qt works, if we want a decent look on windows, we have to do some specific tweaks there.
    if cfg!(target_os = "windows") {
        unsafe { widget_layout.as_mut().unwrap().set_contents_margins((2, 2, 2, 2)) };
        unsafe { widget_layout.as_mut().unwrap().set_spacing(1) }; 
    }
    else {
        unsafe { widget_layout.as_mut().unwrap().set_contents_margins((0, 0, 0, 0)) };
        unsafe { widget_layout.as_mut().unwrap().set_spacing(0) };           
    }

    widget_layout
}

/// This function creates the stylesheet used for the dark theme in windows.
pub fn create_dark_theme_stylesheet() -> String {
    format!("
        /* Normal buttons, with no rounded corners, dark background (darker when enabled), and colored borders. */

        QPushButton {{
            border-style: solid;
            border-width: 1px;
            padding-top: 5px;
            padding-bottom: 4px;
            padding-left: 10px;
            padding-right: 10px;
            border-color: #{button_bd_off};
            color: #{text_normal};
            background-color: #{button_bg_off};
        }}
        QPushButton:hover {{
            border-color: #{button_bd_hover};
            color: #{text_highlighted};
            background-color: #{button_bg_hover};
        }}
        QPushButton:pressed {{
            border-color: #{button_bd_hover};
            color: #{text_highlighted};
            background-color: #{button_bg_on};
        }}
        QPushButton:checked {{
            border-color: #{button_bd_hover};
            background-color: #{button_bg_on};
        }}
        QPushButton:disabled {{
            color: #808086;
            background-color: #{button_bg_off};
        }}

        /* Normal checkboxes */
        QCheckBox::indicator:unchecked {{
            border-style: solid;
            border-width: 1px;
            border-color: #{checkbox_bd_off};
        }}
        /* Disabled due to the evanesce check bug.
        QCheckBox::indicator:checked {{
            height: 12px;
            width: 12px;
            border-style: solid;
            border-width: 1px;
            border-color: #{checkbox_bd_off};
            image:url(img/checkbox_check.png);
        }}
        QCheckBox::indicator:hover {{
            border-style: solid;
            border-width: 1px;
            border-color: #{checkbox_bd_hover};
        }}
        */

        /* Tweaked TableView, so the Checkboxes are white and easy to see. */

        /* Checkboxes */                    
        QTableView::indicator:unchecked {{
            border-style: solid;
            border-width: 1px;
            border-color: #{checkbox_bd_off};
        }}

        /* Disabled due to the evanesce check bug.
        QTableView::indicator:hover {{
            border-style: solid;
            border-width: 1px;
            border-color: #{checkbox_bd_hover};
        }}
        QTableView::indicator:checked {{
            border-style: solid;
            border-width: 1px;
            border-color: #{checkbox_bd_off};
            image:url(img/checkbox_check.png);
        }}
        */
        /* Normal LineEdits, with no rounded corners, dark background (darker when enabled), and colored borders. */

        QLineEdit {{
            border-style: solid;
            border-width: 1px;
            padding-top: 3px;
            padding-bottom: 3px;
            padding-left: 3px;
            padding-right: 3px;
            border-color: #{button_bd_off};
            color: #{text_normal};
            background-color: #{button_bg_off};
        }}
        QLineEdit:hover {{
            border-color: #{button_bd_hover};
            color: #{text_highlighted};
            background-color: #{button_bg_hover};
        }}

        QLineEdit:disabled {{
            color: #808086;
            background-color: #{button_bg_off};
        }}

        /* Combos, similar to buttons. */

        QComboBox {{
            border-style: solid;
            border-width: 1px;
            padding-top: 3px;
            padding-bottom: 3px;
            padding-left: 10px;
            padding-right: 10px;
            border-color: #{button_bd_off};
            color: #{text_normal};
            background-color: #{button_bg_off};
        }}

        /* TreeView, with no rounded corners and darker. */
        QTreeView {{
            border-style: solid;
            border-width: 1px;
            border-color: #{button_bd_off};
        }}

        ", 
        button_bd_hover = *ORANGE,
        button_bd_off = *SLIGHTLY_DARKER_GREY,
        button_bg_on = *SLIGHTLY_DARKER_GREY,
        button_bg_off = *MEDIUM_DARKER_GREY,
        button_bg_hover = *DARK_GREY,
        text_normal = *KINDA_WHITY_GREY,
        text_highlighted = *EVEN_MORE_WHITY_GREY,

        checkbox_bd_off = *KINDA_WHITY_GREY,
        checkbox_bd_hover = *ORANGE
    )
}