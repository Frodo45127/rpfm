// In this file are all the helper functions used by the UI (mainly GTK here)
extern crate num;
extern crate gdk_pixbuf;

use self::gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use gtk::{
    MessageDialog, TreeStore, TreeSelection, TreeView, TreePath, Rectangle, Label, Justification,
    Grid, Statusbar, MessageType, ButtonsType, DialogFlags, ApplicationWindow, ResponseType,
    AboutDialog, License, WindowPosition
};
use std::cmp::Ordering;
use std::path::PathBuf;
use std::fmt::Display;

use packfile::packfile::PackFile;

pub mod packedfile_db;
pub mod packedfile_loc;
pub mod packedfile_text;
pub mod packedfile_image;
pub mod packedfile_rigidmodel;
pub mod settings;
pub mod updater;

//----------------------------------------------------------------------------//
//             UI Creation functions (to build the UI on start)
//----------------------------------------------------------------------------//

/// This function creates an `AboutDialog` with all the credits, logo, license... done, and returns it.
pub fn create_about_window(
    version: &str,
    rpfm_path: &PathBuf,
    parent_window: &ApplicationWindow
) -> AboutDialog {

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

    // Return the `AboutDialog`.
    about_dialog
}


//----------------------------------------------------------------------------//
//              Utility functions (helpers and stuff like that)
//----------------------------------------------------------------------------//

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

/// This function shows a Dialog window with some text. For notification of success and errors.
/// It requires:
/// - dialog: &MessageDialog object. It's the dialog windows we are going to use.
/// - text: something that implements the trait "Display", so we want to put in the dialog window.
pub fn show_dialog<T: Display>(dialog: &MessageDialog, text: T) {
    dialog.set_property_secondary_text(Some(&text.to_string()));
    dialog.run();
    dialog.hide_on_delete();
}

/// This function shows a message in the Statusbar. For notification of common errors and low
/// importance stuff. It requires:
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

/// This function recreates the entire TreeView (as it's a pain to update it) and expand the selected
/// TreePath, or the first parent (the PackFile) if there is nothing selected.
/// It requires:
/// - folder_tree_store: &TreeStore that the TreeView uses.
/// - mut pack_file_decoded: &mut PackFile we have opened, to get the data for the TreeView.
/// - folder_tree_selection: &TreeSelection, if there is something selected when we run this.
/// - folder_tree_view: &TreeView to update.
/// - climb_to_parent: True if we want to expand the parent of the selection, not the selection itself.
pub fn update_tree_view_expand_path(
    folder_tree_store: &TreeStore,
    pack_file_decoded: &PackFile,
    folder_tree_selection: &TreeSelection,
    folder_tree_view: &TreeView,
    climb_to_parent: bool
) {

    // We get the currently selected path in indices (a Vec<i32>). If there is nothing selected,
    // we get the first iter instead (the PackFile).
    let mut tree_path_index: Vec<i32> = vec![];
    let tree_path_index_selected: Vec<i32>;
    let selected_path = folder_tree_selection.get_selected();
    match selected_path {
        Some(path) => {
            tree_path_index = path.0.get_path(&path.1).unwrap().get_indices();
            tree_path_index_selected = tree_path_index.to_vec();
            if tree_path_index.len() > 1 && climb_to_parent {
                tree_path_index.pop();
            }
        }
        None => {
            tree_path_index.push(0);
            tree_path_index_selected = tree_path_index.to_vec();
        }

    }

    // Then we update the TreeView with all the data and expand the path we got before.
    update_tree_view(folder_tree_store, pack_file_decoded);
    folder_tree_view.expand_to_path(&TreePath::new_from_indicesv(&tree_path_index));
    folder_tree_selection.select_path(&TreePath::new_from_indicesv(&tree_path_index_selected));
}

/// This function clears the current TreeView, takes all the data needed for the new one, sort it
/// properly (folder -> file, A -> Z), takes care of duplicates and push it to the TreeStore so it's
/// displayed in the TreeView.
/// It requires:
/// - folder_tree_store: &TreeStore that the TreeView uses.
/// - pack_file_decoded: &PackFile we have opened, to get the data for the TreeView.
pub fn update_tree_view(
    folder_tree_store: &TreeStore,
    pack_file_decoded: &PackFile
){

    // First, we clean the TreeStore
    folder_tree_store.clear();

    // Second, we set as the big_parent, the base for the folders of the TreeView, a fake folder
    // with the name of the PackFile.
    let big_parent = folder_tree_store.insert_with_values(None, None, &[0], &[&format!("{}", pack_file_decoded.pack_file_extra_data.file_name)]);

    // Third, we get all the paths of the PackedFiles inside the Packfile in a Vector.
    let mut sorted_path_list = vec![];
    for i in &pack_file_decoded.pack_file_data.packed_files {
        sorted_path_list.push(&i.packed_file_path);
    }

    // Fourth, we sort that vector using this horrific monster I don't want to touch again using
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
}
