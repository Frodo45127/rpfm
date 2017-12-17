// In this file are all the helper functions used by the UI (mainly GTK here)
extern crate num;

use gtk::prelude::*;
use gtk::{
    MessageDialog, TreeStore, TreeSelection, TreeView, TreePath, Rectangle, Box, Label
};
use std::cmp::Ordering;
use std::path::PathBuf;

use packfile::packfile::PackFile;

pub mod packedfile_loc;
pub mod packedfile_db;

/// This function shows a Message in the specified Box.
pub fn display_help_tips(packed_file_data_display: &Box) {
    let tips = format!(
        "Welcome to Rusted PackFile Manager! Here you have some tips on how to use it:
        \n
        - You can open a PackFile by dragging it to the big PackFile TreeView (where the PackFile\'s files appear when you open it).\n
        - You can rename anything (even the PackFile) by double-clicking it.\n
        - You can insta-patch your siege maps (if you're a mapper) with the \"Patch SiegeAI\" feature from the \"Special Stuff\" menu.\n
        "
    );

    let packed_file_text_view_label: Label = Label::new(Some(&*tips));

    packed_file_data_display.pack_start(&packed_file_text_view_label, true, true, 0);
    packed_file_data_display.show_all();
}

/// This function shows a Dialog window with some text. For notification of success and errors.
/// It requires:
/// - dialog: &MessageDialog object. It's the dialog windows we are going to use.
/// - text: String, the text we want to put in the dialog window.
pub fn show_dialog(dialog: &MessageDialog, text: String) {
    dialog.set_property_secondary_text(Some(text.as_str()));
    dialog.run();
    dialog.hide_on_delete();
}

/// This function get the rect needed to put the popovers in the correct places when we create them,
/// all of this thanks to the magic of the FileChooserDialog from GTK3.
/// It requires:
/// - folder_tree_view: The TreeView we are going to use as parent of the Popover.
/// - cursor_position: An option(f64, f64). This is usually get using gdk::EventButton::get_position
/// or something like that. In case we aren't using a button, we just put None and get a default position.
pub fn get_rect_for_popover(
    folder_tree_view: &TreeView,
    cursor_position: Option<(f64, f64)>
) -> Rectangle {
    let cell = folder_tree_view.get_cursor();
    let mut rect: Rectangle;
    if let Some(_) = cell.0.clone() {
        rect = folder_tree_view.get_cell_area(
            Some(&cell.0.unwrap()),
            Some(&cell.1.unwrap())
        );
    }
    else {
        rect = folder_tree_view.get_cell_area(
            None,
            None
        );
    }

    let rect_new_coords: (i32, i32) = folder_tree_view.convert_bin_window_to_widget_coords(rect.x, rect.y);
    rect.y = rect_new_coords.1;
    match cursor_position {
        Some(cursor_pos) =>  rect.x = num::clamp((cursor_pos.0 as i32) - 20, 0, folder_tree_view.get_allocated_width() - 40),
        None => rect.x = num::clamp(rect.x, 0, folder_tree_view.get_allocated_width() - 40),
    }
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
            let mut filtered_path: Vec<String> = file_path.to_str().unwrap().to_string().split("/").map(|s| s.to_string()).collect();
            tree_path.append(&mut filtered_path);
        }
        else {
            let mut filtered_path: Vec<String> = file_path.to_str().unwrap().to_string().split("\\").map(|s| s.to_string()).collect();
            tree_path.append(&mut filtered_path);
        }
        tree_path.reverse();
    }

    // Then we get the selected path, reverse it, append it to the current
    // path, and reverse it again. That should give us the full tree_path in the form we need it.
    let mut tree_path_from_selection = get_tree_path_from_selection(&folder_tree_selection, false);
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
/// - is_for_renaming: bool. True when we use this to get a path of something we want to rename.
pub fn get_tree_path_from_selection(
    folder_tree_selection: &TreeSelection,
    is_for_renaming: bool
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

        // If we are renaming, we need to keep the name of the PackFile, as we can rename that too.
        if !is_for_renaming {
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
    let selected_path = folder_tree_selection.get_selected();
    match selected_path {
        Some(path) => {
            tree_path_index = path.0.get_path(&path.1).unwrap().get_indices();
            if tree_path_index.len() > 1 && climb_to_parent {
                tree_path_index.pop();
            }
        }
        None => {
            tree_path_index.push(0);
        }

    }

    // Then we update the TreeView with all the data and expand the path we got before.
    update_tree_view(&folder_tree_store, &pack_file_decoded);
    folder_tree_view.expand_to_path(&TreePath::new_from_indicesv(&tree_path_index));

}

/// This function clears the current TreeView, takes all the data needed for the new one, sort it
/// properly (folder -> file, A -> Z), takes care of duplicates and push it to the TreeStore so it's
/// displayed in the TreeView.
/// It requires:
/// - folder_tree_store: &TreeStore that the TreeView uses.
/// - pack_file_decoded: &mut PackFile we have opened, to get the data for the TreeView.
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
        let mut result = Ordering::Greater;
        let ordered = false;
        while !ordered {

            // If both options have the same name.
            if a[index] == b[index] {

                // If A doesn't have more childrens, but B has them, A is a file and B a folder.
                if index == (a.len() - 1) && index < (b.len() - 1) {
                    result = Ordering::Greater;
                    break;
                }

                // If B doesn't have more childrens, but A has them, B is a file and A a folder.
                else if index < (a.len() - 1) && index == (b.len() - 1) {
                    result = Ordering::Less;
                    break;
                }

                // If both options still has childrens, continue the loop.
                else if index < (a.len() - 1) && index < (b.len() - 1) {
                    index += 1;
                    continue;
                }
                else {
                    panic!("This should never happen, but I'll left this here, just in case.");
                }

            // If both options are different.
            } else {

                // If both have no more childrens, both are files.
                if index == (a.len() - 1) && index == (b.len() - 1) {
                    result = a.cmp(b);
                    break;
                }

                // If both have more childrens, both are folders
                else if index < (a.len() - 1) && index < (b.len() - 1) {
                    result = a.cmp(b);
                    break;

                }

                // If A doesn't have more childrens, but B has them, A is a file and B a folder.
                else if index == (a.len() - 1) && index < (b.len() - 1) {
                    result = Ordering::Greater;
                    break;

                }
                // If B doesn't have more childrens, but A has them, B is a file and A a folder.
                else if index < (a.len() - 1) && index == (b.len() - 1) {
                    result = Ordering::Less;
                    break;
                }
                else {
                    panic!("This should never happen, but I'll left this here, just in case.");
                }
            }
        }
        result
    });

    // Once we get the entire path list sorted, we add the paths to the TreeStore one by one,
    // skipping duplicate entries.
    for i in sorted_path_list.iter() {

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
            // so we check all his childrens. If any of them is equal to the current folder we are
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
                        if folder_tree_store.iter_next(&current_child) == false {
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