// In this file are all the helper functions used by the UI (mainly GTK here)
extern crate num;
extern crate gdk_pixbuf;

use self::gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use gtk::{
    MessageDialog, TreeStore, TreeSelection, TreeView, Rectangle, Label, Justification,
    Grid, Statusbar, MessageType, ButtonsType, DialogFlags, ApplicationWindow, ResponseType,
    AboutDialog, License, WindowPosition, TreeIter, Application, Paned, Orientation, CellRendererMode,
    TreeViewColumn, CellRendererText, ScrolledWindow
};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::fmt::Display;

use common::*;
use packfile::packfile::PackFile;

pub mod packedfile_db;
pub mod packedfile_loc;
pub mod packedfile_text;
pub mod packedfile_image;
pub mod packedfile_rigidmodel;
pub mod settings;
pub mod updater;

/// This struct is what we return to create the main window at the start of the program.
pub struct MainWindow {

    // Main window.
    pub window: ApplicationWindow,

    // This is the box where all the PackedFile Views are created.
    pub packed_file_data_display: Grid,

    // Status bar at the bottom of the program. To show informative messages.
    pub status_bar: Statusbar,

    // TreeView used to see the PackedFiles, and his TreeStore and TreeSelection.
    pub folder_tree_view: TreeView,
    pub folder_tree_store: TreeStore,
    pub folder_tree_selection: TreeSelection,

    // Column and cells for the `TreeView`.
    pub folder_tree_view_cell: CellRendererText,
    pub folder_tree_view_column: TreeViewColumn,
}

//----------------------------------------------------------------------------//
//             UI Creation functions (to build the UI on start)
//----------------------------------------------------------------------------//

/// Implementation of `MainWindow`.
impl MainWindow {

    /// This function builds the Main Window at the start of the program. Because Glade is too buggy to use it.
    pub fn create_main_window(application: &Application, rpfm_path: &PathBuf) -> Self {

        // Create the main `ApplicationWindow`.
        let window = ApplicationWindow::new(application);
        window.set_position(WindowPosition::Center);
        window.set_title("Rusted PackFile Manager");

        // Config the icon for the main window. If this fails, something went wrong when setting the paths,
        // so crash the program, as we don't know what more is broken.
        window.set_icon_from_file(&Path::new(&format!("{}/img/rpfm.png", rpfm_path.to_string_lossy()))).unwrap();

        // Create the `Grid` that'll hold everything except the top `MenuBar`.
        let main_grid = Grid::new();
        main_grid.set_border_width(6);
        main_grid.set_row_spacing(3);
        main_grid.set_column_spacing(3);

        // Attach it to the main window.
        window.add(&main_grid);

        // Create the `Paned`.
        let paned = Paned::new(Orientation::Horizontal);
        paned.set_position(350);
        paned.set_wide_handle(true);
        paned.set_size_request(1100, 350);

        // Create the `ScrolledWindow` for the `TreeView`.
        let folder_scroll = ScrolledWindow::new(None, None);
        folder_scroll.set_hexpand(true);
        folder_scroll.set_vexpand(true);

        // Create the `TreeView`.
        let folder_tree_view = TreeView::new();
        let folder_tree_store = TreeStore::new(&[String::static_type()]);
        let folder_tree_selection = folder_tree_view.get_selection();

        // Add the `TreeView` to the `ScrolledWindow`.
        folder_scroll.add(&folder_tree_view);

        // Config stuff for the `TreeView`.
        folder_tree_view.set_model(Some(&folder_tree_store));

        let folder_tree_view_column = TreeViewColumn::new();
        let folder_tree_view_cell = CellRendererText::new();
        folder_tree_view_cell.set_property_editable(true);
        folder_tree_view_cell.set_property_mode(CellRendererMode::Activatable);
        folder_tree_view_column.pack_start(&folder_tree_view_cell, true);
        folder_tree_view_column.add_attribute(&folder_tree_view_cell, "text", 0);

        folder_tree_view.append_column(&folder_tree_view_column);
        folder_tree_view.set_margin_bottom(10);
        folder_tree_view.set_enable_search(false);
        folder_tree_view.set_search_column(0);
        folder_tree_view.set_activate_on_single_click(true);
        folder_tree_view.set_headers_visible(false);
        folder_tree_view.set_enable_tree_lines(true);

        // Create the data `Grid`.
        let packed_file_data_display = Grid::new();

        // Attach them to the `Paned`.
        paned.add1(&folder_scroll);
        paned.add2(&packed_file_data_display);

        // Create the `Statusbar`.
        let status_bar = Statusbar::new();
        status_bar.set_margin_bottom(0);
        status_bar.set_margin_top(0);
        status_bar.set_margin_start(0);
        status_bar.set_margin_end(0);

        // Attach the `Paned` and the `Statusbar` to the main `Grid`.
        main_grid.attach(&paned, 0, 0, 1, 1);
        main_grid.attach(&status_bar, 0, 1, 1, 1);

        // Return the `MainWindow` struct.
        Self {

            // Main window.
            window,

            // This is the box where all the PackedFile Views are created.
            packed_file_data_display,

            // Status bar at the bottom of the program. To show informative messages.
            status_bar,

            // TreeView used to see the PackedFiles, and his TreeStore and TreeSelection.
            folder_tree_view,
            folder_tree_store,
            folder_tree_selection,

            // Column and cells for the `TreeView`.
            folder_tree_view_cell,
            folder_tree_view_column,
        }
    }
}

/// This function creates an `AboutDialog` with all the credits, logo, license... done, and shows it.
pub fn show_about_window(
    version: &str,
    rpfm_path: &PathBuf,
    parent_window: &ApplicationWindow
) {

    // Create the `AboutDialog`.
    let about_dialog = AboutDialog::new();

    // Configure the `AboutDialog` with all our stuff.
    about_dialog.set_program_name("Rusted PackFile Manager");
    about_dialog.set_version(version);
    about_dialog.set_license_type(License::MitX11);
    about_dialog.set_website("https://github.com/Frodo45127/rpfm");
    about_dialog.set_website_label("Source code and more info here :)");
    about_dialog.set_comments(Some("Made by modders, for modders."));

    // Config the icon for the "About" window. If this fails, something went wrong when setting the paths,
    // so crash the program, as we don't know what more is broken.
    let icon_path = PathBuf::from(format!("{}/img/rpfm.png", rpfm_path.to_string_lossy()));
    about_dialog.set_icon_from_file(&icon_path).unwrap();
    about_dialog.set_logo(&Pixbuf::new_from_file(&icon_path).unwrap());

    // Credits stuff.
    about_dialog.add_credit_section("Created and Programmed by", &["Frodo45127"]);
    about_dialog.add_credit_section("Icon by", &["Maruka"]);
    about_dialog.add_credit_section("RigidModel research by", &["Mr.Jox", "Der Spaten", "Maruka", "Frodo45127"]);
    about_dialog.add_credit_section("Windows's theme", &["\"Materia for GTK3\" by nana-4"]);
    about_dialog.add_credit_section("Special thanks to", &["- PFM team (for providing the community\n   with awesome modding tools).", "- CA (for being a mod-friendly company)."]);

    // Center the `AboutDialog` in the middle of the screen.
    about_dialog.set_position(WindowPosition::Center);

    // Give a father to the poor orphan...
    about_dialog.set_transient_for(parent_window);

    // Run the `AboutDialog`.
    about_dialog.run();
    about_dialog.destroy();
}

//----------------------------------------------------------------------------//
//              Utility functions (helpers and stuff like that)
//----------------------------------------------------------------------------//

/// This enum has the different possible operations we want to do over a `TreeView`. The options are:
/// - Build: Build the entire `TreeView` from nothing.
/// - Add: Add a File/Folder to the `TreeView`. Requires the path in the `TreeView`, without the mod's name.
/// - AddFromPackFile: Add a File/Folder from another `TreeView`. Requires `source_path`, `destination_path`, the extra `TreeStore` and the extra `TreeSelection`.
/// - Delete: Remove a File/Folder from the `TreeView`.
/// - Rename: Change the name of a File/Folder from the TreeView. Requires the new name.
#[derive(Clone, Debug)]
pub enum TreeViewOperation {
    Build,
    Add(Vec<String>),
    AddFromPackFile(Vec<String>, Vec<String>, Vec<Vec<String>>),
    Delete,
    Rename(String),
}

/// This function shows a Message in the specified Grid.
pub fn display_help_tips(packed_file_data_display: &Grid) {
    let tips = "Welcome to Rusted PackFile Manager! Here you have some tips on how to use it:
        - You can rename anything (except the PackFile) by double-clicking it.
        - You can open a PackFile by dragging it to the big PackFile Tree View.
        - To patch an Attila model to work in Warhammer, select it and press \"Patch to Warhammer 1&2\".
        - You can insta-patch your siege maps (if you're a mapper) with the \"Patch SiegeAI\" feature from the \"Special Stuff\" menu.";

    let packed_file_text_view_label: Label = Label::new(Some(tips));
    packed_file_text_view_label.set_justify(Justification::Left);

    packed_file_data_display.attach(&packed_file_text_view_label, 0, 0, 1, 1);
    packed_file_data_display.show_all();
}

/// This function shows a "Success" or "Error" Dialog with some text. For notification of success and
/// high importance errors.
/// It requires:
/// - parent_window: a reference to the `Window` that'll act as "parent" of the dialog.
/// - is_success: true for "Success" Dialog, false for "Error" Dialog.
/// - text: something that implements the trait "Display", so we want to put in the dialog window.
pub fn show_dialog<T: Display>(parent_window: &ApplicationWindow, is_success: bool, text: T) {

    // Depending on the type of the dialog, set everything specific here.
    let title = if is_success { "Success!" } else { "Error!" };
    let message_type = if is_success { MessageType::Info } else { MessageType::Error };

    // Create the dialog...
    let dialog = MessageDialog::new(
        Some(parent_window),
        DialogFlags::from_bits(1).unwrap(),
        message_type,
        ButtonsType::Ok,
        &title
    );

    // Set the title and secondary text.
    dialog.set_title(&title);
    dialog.set_property_secondary_text(Some(&text.to_string()));

    // Run & Destroy the Dialog.
    dialog.run();
    dialog.destroy();
}

/// This function shows a message in the Statusbar. For notification of common errors and low
/// importance stuff. It requires:
/// - status_bar: a reference to the `Statusbar` where to show the message.
/// - text: something that implements the trait "Display", so we want to put in the Statusbar.
pub fn show_message_in_statusbar<T: Display>(status_bar: &Statusbar, message: T) {
    let message = &message.to_string();
    status_bar.push(status_bar.get_context_id(message), message);
}

/// This function shows a message asking for confirmation. For use in operations that implies unsaved
/// data loss.
pub fn are_you_sure(parent_window: &ApplicationWindow, is_modified: bool, is_delete_my_mod: bool) -> bool {

    // If the mod has been modified, create the dialog. Otherwise, return true.
    if is_modified {
        let are_you_sure_dialog = MessageDialog::new(
            Some(parent_window),
            DialogFlags::from_bits(1).unwrap(),
            MessageType::Error,
            ButtonsType::None,
            "Are you sure?"
        );

        are_you_sure_dialog.add_button("Cancel", -6);
        are_you_sure_dialog.add_button("Accept", -3);
        are_you_sure_dialog.set_title("Are you sure?");

        let message = if is_delete_my_mod {
            "You are going to delete this mod from your disk. There is no way to recover it after that."
        } else {
            "There are some changes yet to save."
        };
        are_you_sure_dialog.set_property_secondary_text(Some(message));

        // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
        let response_ok: i32 = ResponseType::Accept.into();

        if are_you_sure_dialog.run() == response_ok {
            are_you_sure_dialog.destroy();
            true
        } else {
            are_you_sure_dialog.destroy();
            false
        }
    } else { true }
}

/// This function get the rect needed to put the popovers in the correct places when we create them,
/// all of this thanks to the magic of the FileChooserDialog from GTK3.
/// It requires:
/// - tree_view: The TreeView we are going to use as parent of the Popover.
/// - cursor_position: An option(f64, f64). This is usually get using gdk::EventButton::get_position
/// or something like that. In case we aren't using a button, we just put None and get a default position.
pub fn get_rect_for_popover(
    tree_view: &TreeView,
    cursor_position: Option<(f64, f64)>
) -> Rectangle {
    let cursor = tree_view.get_cursor();
    let mut rect: Rectangle = if cursor.0.clone().is_some() {

        // If there is a tree_path selected, get the coords of the cursor.
        tree_view.get_cell_area(
            Some(&cursor.0.unwrap()),
            Some(&cursor.1.unwrap())
        )
    }
    else {

        // If there is no tree_path selected, it sets the coords to 0,0.
        tree_view.get_cell_area(
            None,
            None
        )
    };

    // Replace the rect.x with the widget one, so it's not crazy when you scroll to the size.
    rect.x = tree_view.convert_tree_to_widget_coords(rect.x, rect.y).0;

    // If the TreeView has headers, fix the Y coordinate too.
    if tree_view.get_headers_visible() {

        // FIXME: This needs to be get programatically, as it just work with font size 10.
        rect.y += 32; // 32 - height of the header.
    }

    // If we got a precise position of the cursor, we get the exact position of the x, based on the
    // current x we have. This is partly black magic.
    if let Some(cursor_pos) = cursor_position {
        let widget_coords = tree_view.convert_tree_to_widget_coords(cursor_pos.0 as i32, cursor_pos.1 as i32);
        rect.x = num::clamp((widget_coords.0 as i32) - 20, 0, tree_view.get_allocated_width() - 40);
    }

    // Set the witdth to 40 (more black magic?) and return the rect.
    rect.width = 40;
    rect
}

/// This function is used to get the complete TreePath (path in a GTKTreeView) of an external file
/// or folder in a Vec<String> format. Needed to get the path for the TreeView and for encoding
/// the file in a PackFile.
/// It requires:
/// - file_path: &PathBuf of the external file.
/// - folder_tree_selection: &TreeSelection of the place of the TreeView where we want to add the file.
/// - is_file: bool. True if the &PathBuf is from a file, false if it's a folder.
pub fn get_tree_path_from_pathbuf(
    file_path: &PathBuf,
    folder_tree_selection: &TreeSelection,
    is_file: bool
) -> Vec<String> {

    let mut tree_path: Vec<String> = vec![];

    // If it's a single file, we get his name and push it to the tree_path vector.
    if is_file {
        tree_path.push(file_path.file_name().expect("error, nombre no encontrado").to_str().unwrap().to_string());
    }

    // If it's a folder, we filter his PathBuf, turn it into Vec<String> and push it to the tree_path.
    // After that, we reverse the vector, so it's easier to create the full tree_path from it.
    else {
        if cfg!(target_os = "linux") {
            let mut filtered_path: Vec<String> = file_path.to_str().unwrap().to_string().split('/').map(|s| s.to_string()).collect();
            tree_path.append(&mut filtered_path);
        }
        else {
            let mut filtered_path: Vec<String> = file_path.to_str().unwrap().to_string().split('\\').map(|s| s.to_string()).collect();
            tree_path.append(&mut filtered_path);
        }
        tree_path.reverse();
    }

    // Then we get the selected path, reverse it, append it to the current
    // path, and reverse it again. That should give us the full tree_path in the form we need it.
    let mut tree_path_from_selection = get_tree_path_from_selection(folder_tree_selection, false);
    tree_path_from_selection.reverse();
    tree_path.append(&mut tree_path_from_selection);
    tree_path.reverse();

    // Return the tree_path (from parent to children)
    tree_path
}

/// This function is used to get the complete TreePath (path in a GTKTreeView) of a selection of the
/// TreeView. I'm sure there are other ways to do it, but the TreeView has proven to be a mystery
/// BEYOND MY COMPREHENSION, so we use this for now.
/// It requires:
/// - folder_tree_selection: &TreeSelection of the place of the TreeView we want to know his TreePath.
/// - include_packfile: bool. True if we want the TreePath to include the PackFile's name.
pub fn get_tree_path_from_selection(
    folder_tree_selection: &TreeSelection,
    include_packfile: bool
) -> Vec<String>{

    let mut tree_path: Vec<String> = vec![];

    // We create the full tree_path from the tree_path we have and the TreePath of the folder
    // selected in the TreeView (adding the new parents at the end of the vector, and then
    // reversing the vector).
    if let Some((model, iter)) = folder_tree_selection.get_selected() {
        let mut me = iter;
        let mut path_completed = false;
        while !path_completed {
            tree_path.push(model.get_value(&me, 0).get().unwrap());
            match model.iter_parent(&me) {
                Some(parent) => {
                    me = parent;
                    path_completed = false;
                },
                None => path_completed = true,
            };
        }

        // We only want to keep the name of the PackFile on specific situations.
        if !include_packfile {
            tree_path.pop();
        }
        tree_path.reverse();
    }

    // Return the tree_path (from parent to children)
    tree_path
}

/// This function is used to get the complete Path (path in a GTKTreeView) of a `TreeIter` of the
/// TreeView. I'm sure there are other ways to do it, but the TreeView has proven to be a mystery
/// BEYOND MY COMPREHENSION, so we use this for now.
/// It requires:
/// - tree_iter: &TreeIter of the TreeView we want to know his Path.
/// - tree_store: &TreeStore of our TreeView.
/// - include_packfile: bool. True if we want the Path to include the PackFile's name.
pub fn get_path_from_tree_iter(
    tree_iter: &TreeIter,
    tree_store: &TreeStore,
    include_packfile: bool
) -> Vec<String> {

    let mut tree_path: Vec<String> = vec![];

    // We create the full tree_path from the tree_path we have and the TreePath of the folder
    // selected in the TreeView (adding the new parents at the end of the vector, and then
    // reversing the vector).
    let mut me = tree_iter.clone();
    let mut path_completed = false;
    while !path_completed {
        tree_path.push(tree_store.get_value(&me, 0).get().unwrap());
        match tree_store.iter_parent(&me) {
            Some(parent) => {
                me = parent.clone();
                path_completed = false;
            },
            None => path_completed = true,
        };
    }

    // We only want to keep the name of the PackFile on specific situations.
    if !include_packfile {
        tree_path.pop();
    }
    tree_path.reverse();

    // Return the tree_path (from parent to children)
    tree_path
}

/// This function updates the provided `TreeView`, depending on the operation we want to do.
/// It requires:
/// - folder_tree_store: `&TreeStore` that the `TreeView` uses.
/// - mut pack_file_decoded: `&mut PackFile` we have opened, to get the data for the `TreeView`.
/// - folder_tree_selection: `&TreeSelection`, if there is something selected when we run this.
/// - operation: the `TreeViewOperation` we want to realise.
/// - type: the type of whatever is selected.
pub fn update_treeview(
    folder_tree_store: &TreeStore,
    pack_file_decoded: &PackFile,
    folder_tree_selection: &TreeSelection,
    operation: TreeViewOperation,
    selection_type: TreePathType,
) {

    // We act depending on the operation requested.
    match operation {

        // If we want to build a new TreeView...
        TreeViewOperation::Build => {

            // FIXME: This CTDs RPFM due to changing the cursor... which triggers a callback that tries to borrow
            // the packfile already borrowed here. For now I've forced the clear to trigger before calling this
            // function when we do a `TreeViewOperation::Build`, but that's a dirty hack and need to be fixed.
            //
            // First, we clean the TreeStore
            //folder_tree_store.clear();

            // Second, we set as the big_parent, the base for the folders of the TreeView, a fake folder
            // with the name of the PackFile. All big things start with a lie.
            let big_parent = folder_tree_store.insert_with_values(None, None, &[0], &[&format!("{}", pack_file_decoded.pack_file_extra_data.file_name)]);

            // Third, we get all the paths of the PackedFiles inside the Packfile in a Vector.
            let mut sorted_path_list = vec![];
            for i in &pack_file_decoded.pack_file_data.packed_files {
                sorted_path_list.push(&i.packed_file_path);
            }

            // Fourth, we sort that vector using this horrific monster I don't want to touch again, using
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

            // Once we get the entire path list sorted, we add the paths to the TreeStore one by one,
            // skipping duplicate entries.
            for i in &sorted_path_list {

                // First, we reset the parent to the big_parent (the PackFile).
                let mut parent = big_parent.clone();

                // Then, we form the path ("parent -> child" style path) to add to the TreeStore.
                for j in i.iter() {

                    // If it's the last string in the file path, it's a file, so we add it to the TreeStore.
                    if j == i.last().unwrap() {
                        parent = folder_tree_store.insert_with_values(Some(&parent), None, &[0], &[&format!("{}", j)]);
                    }

                    // If it's a folder, we check first if it's already in the TreeStore using the following
                    // logic:
                    // If the current parent has a child, it should be a folder already in the TreeStore,
                    // so we check all his children. If any of them is equal to the current folder we are
                    // trying to add and it has at least one child, it's a folder exactly like the one we are
                    // trying to add, so that one becomes our new parent. If there is no equal folder to
                    // the one we are trying to add, we add it, turn it into the new parent, and repeat.
                    else {
                        let mut duplicate_found = false;
                        if folder_tree_store.iter_has_child(&parent) {
                            let mut no_more_childrens = false;
                            let current_child = folder_tree_store.iter_children(&parent).unwrap();
                            while !no_more_childrens {
                                let current_child_text: String = folder_tree_store.get_value(&current_child, 0).get().unwrap();
                                if &current_child_text == j && folder_tree_store.iter_has_child(&current_child) {
                                    parent = current_child;
                                    duplicate_found = true;
                                    break;
                                }
                                if !folder_tree_store.iter_next(&current_child) {
                                    no_more_childrens = true;
                                }
                            }
                            if duplicate_found {
                                continue;
                            } else {
                                parent = folder_tree_store.insert_with_values(Some(&parent), None, &[0], &[&format!("{}", j)]);
                            }
                        } else {
                            parent = folder_tree_store.insert_with_values(Some(&parent), None, &[0], &[&format!("{}", j)]);
                        }
                    }
                }
            }
        },

        // If we want to add a file/folder to the `TreeView`...
        TreeViewOperation::Add(path) => {

            // If we got the `TreeIter` for the PackFile...
            if let Some(mut tree_iter) = folder_tree_store.get_iter_first() {

                // Index, to know what field of `path` use in each iteration.
                let mut index = 0;

                // Initiate an endless loop of space and time...
                loop {

                    // If we are using the last thing in the path, it's a file. Otherwise, is a folder.
                    let new_type = if path.len() - 1 == index { TreePathType::File((vec![String::new()],1)) } else { TreePathType::Folder(vec![String::new()]) };

                    // If the current `TreeIter` has a child...
                    if folder_tree_store.iter_has_child(&tree_iter) {

                        // Move our test `TreeIter` to his first child.
                        let mut tree_iter_test = folder_tree_store.iter_children(&tree_iter).unwrap();

                        // Variable to know when to finish the next loop.
                        let mut childs_looped = true;

                        // Loop through all the childs to see if what we want to add already exists.
                        while childs_looped {

                            // Get the current iter's text.
                            let current_iter_text: String = folder_tree_store.get_value(&tree_iter_test, 0).get().unwrap();

                            // If it's the same that we want to add...
                            if current_iter_text == path[index] {

                                // Get both types.
                                let current_path = get_path_from_tree_iter(&tree_iter_test, &folder_tree_store, true);
                                let current_type = get_type_of_selected_tree_path(&current_path, &pack_file_decoded);

                                // And both are files...
                                if current_type == TreePathType::File((vec![String::new()],1)) && new_type == TreePathType::File((vec![String::new()],1)) {

                                    // We run away...
                                    break;
                                }

                                // If both are folder...
                                else if current_type == TreePathType::Folder(vec![String::new()]) && new_type == TreePathType::Folder(vec![String::new()]) {

                                    // Move to that folder.
                                    tree_iter = tree_iter_test.clone();

                                    // Increase the Index.
                                    index += 1;

                                    // And run away...
                                    break;
                                }
                            }

                            // If there is no more childs...
                            if !folder_tree_store.iter_next(&tree_iter_test) {

                                // Stop the loop.
                                childs_looped = false;

                                // Create a new empty child and move to it.
                                tree_iter = folder_tree_store.prepend(&tree_iter);

                                // Set his value.
                                folder_tree_store.set_value(&tree_iter, 0, &path[index].to_value());

                                // Sort properly the `TreeStore` to show the renamed file in his proper place.
                                sort_tree_view(folder_tree_store, pack_file_decoded, new_type.clone(), &tree_iter);

                                // Increase the index.
                                index += 1;
                            }
                        }

                        // If our current type is a File, we reached the end of the path.
                        if new_type == TreePathType::File((vec![String::new()],1)) { break; }

                    }

                    // If it doesn't have a child...
                    else {

                        // Create a new empty child and move to it.
                        tree_iter = folder_tree_store.prepend(&tree_iter);

                        // Set his value.
                        folder_tree_store.set_value(&tree_iter, 0, &path[index].to_value());

                        // Sort properly the `TreeStore` to show the renamed file in his proper place.
                        sort_tree_view(folder_tree_store, pack_file_decoded, new_type.clone(), &tree_iter);

                        // Increase the index.
                        index += 1;

                        // If our current type is a File, we reached the end of the path.
                        if new_type == TreePathType::File((vec![String::new()],1)) { break; }
                    }
                }
            }
        },

        // If we want to add a file/folder from another `TreeView`...
        TreeViewOperation::AddFromPackFile(mut source_prefix, destination_prefix, new_files_list) => {

            // If his path is something...
            if source_prefix.len() > 0 {

                // Take our the last folder.
                source_prefix.pop();

            }

            // For each file...
            for file in new_files_list.iter() {

                // Filter his new path.
                let mut filtered_source_path = file[source_prefix.len()..].to_vec();
                let mut final_path = destination_prefix.to_vec();
                final_path.append(&mut filtered_source_path);

                // Add it to our PackFile's `TreeView`.
                update_treeview(
                    folder_tree_store,
                    pack_file_decoded,
                    folder_tree_selection,
                    TreeViewOperation::Add(final_path),
                    TreePathType::File((vec![String::new()],1)),
                );
            }
        },

        // If we want to delete something from the `TreeView`...
        TreeViewOperation::Delete => {

            // Then we see what type the selected thing is.
            match selection_type {

                // If it's a PackedFile or a Folder...
                TreePathType::File(_) | TreePathType::Folder(_) => {

                    // If we have something selected (just in case)...
                    if let Some(selection) = folder_tree_selection.get_selected() {

                        // We get his `TreeIter`.
                        let mut tree_iter = selection.1;

                        // Begin the endless loop of war and dead.
                        loop {

                            // Get his parent. We can unwrap here because we are never to reach this with a iter without parent.
                            let parent = folder_tree_store.iter_parent(&tree_iter).unwrap();

                            // Remove it from the `TreeStore`.
                            folder_tree_store.remove(&tree_iter);

                            // If the parent has any more childs or it's in root level (PackFile), stop.
                            if folder_tree_store.iter_has_child(&parent) || folder_tree_store.iter_depth(&parent) == 0 {
                                break;
                            }

                            // If we don't have any reason to stop, replace the `tree_iter` with his parent.
                            else {
                                tree_iter = parent;
                            }
                        }
                    }
                }

                // If it's a PackFile...
                TreePathType::PackFile => {

                    // First, we clear the TreeStore.
                    folder_tree_store.clear();

                    // Then we add the PackFile to it. This effectively deletes all the PackedFiles in the PackFile.
                    folder_tree_store.insert_with_values(None, None, &[0], &[&format!("{}", pack_file_decoded.pack_file_extra_data.file_name)]);
                },

                // If we don't have anything selected, we do nothing.
                TreePathType::None => {},
            }
        },

        // If we want to rename something...
        TreeViewOperation::Rename(new_name) => {

            // If we got his `TreeIter`....
            if let Some(selection) = folder_tree_selection.get_selected() {

                // We get our `TreeIter`.
                let mut tree_iter = selection.1;

                // We change the "Name" of the file/folder.
                folder_tree_store.set_value(&tree_iter, 0, &new_name.to_value());

                // Sort properly the `TreeStore` to show the renamed file in his proper place.
                sort_tree_view(folder_tree_store, pack_file_decoded, selection_type, &tree_iter)
            }
        },
    }
}

/// This function is meant to sort newly added items in a `TreeView`. Pls note that the provided
/// `&TreeIter` MUST BE VALID. Otherwise, this can CTD the entire program.
fn sort_tree_view(
    folder_tree_store: &TreeStore,
    pack_file_decoded: &PackFile,
    selection_type: TreePathType,
    tree_iter: &TreeIter
) {

    // Get the previous and next `TreeIter`s.
    let tree_iter_previous = tree_iter.clone();
    let tree_iter_next = tree_iter.clone();

    let iter_previous_exists = folder_tree_store.iter_previous(&tree_iter_previous);
    let iter_next_exists = folder_tree_store.iter_next(&tree_iter_next);

    // If the previous iter is valid, get their path and their type.
    let previous_type = if iter_previous_exists {

        let path_previous = get_path_from_tree_iter(&tree_iter_previous, &folder_tree_store, true);
        get_type_of_selected_tree_path(&path_previous, &pack_file_decoded)
    }

    // Otherwise, return the type as `None`.
    else { TreePathType::None };

    // If the next iter is valid, get their path and their type.
    let next_type = if iter_next_exists {

        let path_next = get_path_from_tree_iter(&tree_iter_next, &folder_tree_store, true);
        get_type_of_selected_tree_path(&path_next, &pack_file_decoded)
    }

    // Otherwise, return the type as `None`.
    else { TreePathType::None };

    // We get the boolean to determinate the direction to move (true -> up, false -> down).
    // If the previous and the next `TreeIter`s are `None`, we don't need to move.
    let direction = if previous_type == TreePathType::None && next_type == TreePathType::None { return }

    // If the top one is `None`, but the bottom one isn't, we go down.
    else if previous_type == TreePathType::None && next_type != TreePathType::None { false }

    // If the bottom one is `None`, but the top one isn't, we go up.
    else if previous_type != TreePathType::None && next_type == TreePathType::None { true }

    // If the top one is a folder, and the bottom one is a file, get the type of our iter.
    else if previous_type == TreePathType::Folder(vec![String::new()]) && next_type == TreePathType::File((vec![String::new()], 1)) {
        if selection_type == TreePathType::Folder(vec![String::new()]) { true } else { false }
    }

    // If the two around it are the same type, compare them and decide.
    else {

        // Get the previous, current and next texts.
        let previous_name: String = folder_tree_store.get_value(&tree_iter_previous, 0).get().unwrap();
        let current_name: String = folder_tree_store.get_value(&tree_iter, 0).get().unwrap();
        let next_name: String = folder_tree_store.get_value(&tree_iter_next, 0).get().unwrap();

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

        // Get the `TreeIter` we want to compare with, depending on our direction.
        let tree_iter_second = tree_iter.clone();
        let iter_second_is_valid = if direction { folder_tree_store.iter_previous(&tree_iter_second) } else { folder_tree_store.iter_next(&tree_iter_second) };

        // If `tree_iter_second` is valid...
        if iter_second_is_valid {

            // Get their path.
            let path_second = get_path_from_tree_iter(&tree_iter_second, &folder_tree_store, true);

            // Get the type of both `TreeIter`.
            let second_type = get_type_of_selected_tree_path(&path_second, &pack_file_decoded);

            // If we have something of the same type than our `TreeIter`...
            if second_type == selection_type {

                // Get the other `TreeIter`s text.
                let second_name: String = folder_tree_store.get_value(&tree_iter_second, 0).get().unwrap();
                let current_name: String = folder_tree_store.get_value(&tree_iter, 0).get().unwrap();

                // Depending on our direction, we sort one way or another
                if direction {

                    // For previous `TreeIter`...
                    let name_list = vec![second_name.to_owned(), current_name.to_owned()];
                    let mut name_list_sorted = vec![second_name.to_owned(), current_name.to_owned()];
                    name_list_sorted.sort();

                    // If the order hasn't changed...
                    if name_list == name_list_sorted {

                        // We are done.
                        break;
                    }

                    // If they have changed positions...
                    else {

                        // We swap them, and update them for the next loop.
                        folder_tree_store.swap(&tree_iter, &tree_iter_second);
                    }

                }

                else {

                    // For next `TreeIter`...
                    let name_list = vec![current_name.to_owned(), second_name.to_owned()];
                    let mut name_list_sorted = vec![current_name.to_owned(), second_name.to_owned()];
                    name_list_sorted.sort();

                    // If the order hasn't changed...
                    if name_list == name_list_sorted {

                        // We are done.
                        break;
                    }

                    // If they have changed positions...
                    else {

                        // We swap them, and update them for the next loop.
                        folder_tree_store.swap(&tree_iter, &tree_iter_second);
                    }

                }
            }

            // If the top one is a File and the bottom one a Folder, it's an special situation. Just swap them.
            else if selection_type == TreePathType::File((vec![String::new()], 1)) && next_type == TreePathType::Folder(vec![String::new()]) {
                folder_tree_store.swap(&tree_iter, &tree_iter_second);
            }

            // If the type is different and it's not an special situation, we can't move anymore.
            else { break; }
        }

        // If the `TreeIter` is invalid, we can't move anymore.
        else { break; }
    }
}
