// In this file we create the UI of the RPFM, and control it (events, updates, etc...).

#![windows_subsystem = "windows"]

extern crate gtk;
extern crate gdk;
extern crate num;

use std::path::PathBuf;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{
    AboutDialog, Builder, MenuItem, Window, WindowPosition, FileChooserDialog,
    TreeView, TreeSelection, TreeStore, MessageDialog,
    CellRendererText, TreeViewColumn, Popover, Entry, CheckMenuItem, Button
};

mod common;
mod ui;
mod pack_file_manager;

// One Function to rule them all, One Function to find them,
// One Function to bring them all and in the darkness bind them.
fn main() {

    // Init GTK3. Boilerplate code.
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    // We import the Glade design and get all the UI objects into variables.
    let glade_design = include_str!("glade/main.glade");
    let builder = Builder::new_from_string(glade_design);

    let window: Window = builder.get_object("gtk_window").expect("Couldn't get gtk_window");

    let window_about: AboutDialog = builder.get_object("gtk_window_about").expect("Couldn't get gtk_window_about");
    let error_dialog: MessageDialog = builder.get_object("gtk_error_dialog").expect("Couldn't get gtk_error_dialog");
    let success_dialog: MessageDialog = builder.get_object("gtk_success_dialog").expect("Couldn't get gtk_success_dialog");
    let rename_popover: Popover = builder.get_object("gtk_rename_popover").expect("Couldn't get gtk_rename_popover");

    let rename_popover_text_entry: Entry = builder.get_object("gtk_rename_popover_text_entry").expect("Couldn't get gtk_rename_popover_text_entry");

    let file_chooser_open_packfile_dialog: FileChooserDialog = builder.get_object("gtk_file_chooser_open_packfile").expect("Couldn't get gtk_file_chooser_open_packfile");
    let file_chooser_save_packfile_dialog: FileChooserDialog = builder.get_object("gtk_file_chooser_save_packfile").expect("Couldn't get gtk_file_chooser_save_packfile");
    let file_chooser_add_file_to_packfile: FileChooserDialog = builder.get_object("gtk_file_chooser_add_file_to_packfile").expect("Couldn't get gtk_file_chooser_add_file_to_packfile");
    let file_chooser_add_folder_to_packfile: FileChooserDialog = builder.get_object("gtk_file_chooser_add_folder_to_packfile").expect("Couldn't get gtk_file_chooser_add_folder_to_packfile");
    let file_chooser_extract_file: FileChooserDialog = builder.get_object("gtk_file_chooser_extract_file").expect("Couldn't get gtk_file_chooser_extract_file");
    let file_chooser_extract_folder: FileChooserDialog = builder.get_object("gtk_file_chooser_extract_folder").expect("Couldn't get gtk_file_chooser_extract_folder");

    let tree_view_add_file: Button = builder.get_object("gtk_context_menu_tree_view_add_file").expect("Couldn't get gtk_context_menu_tree_view_add_file");
    let tree_view_add_folder: Button = builder.get_object("gtk_context_menu_tree_view_add_folder").expect("Couldn't get gtk_context_menu_tree_view_add_folder");
    let tree_view_delete_file: Button = builder.get_object("gtk_context_menu_tree_view_delete_file").expect("Couldn't get gtk_context_menu_tree_view_delete_file");
    let tree_view_extract_file: Button = builder.get_object("gtk_context_menu_tree_view_extract_file").expect("Couldn't get gtk_context_menu_tree_view_extract_file");

    let top_menu_file: MenuItem = builder.get_object("gtk_top_menu_file").expect("Couldn't get gtk_top_menu_file");
    let top_menu_special_stuff: MenuItem = builder.get_object("gtk_top_menu_special_stuff").expect("Couldn't get gtk_top_menu_special_stuff");

    let context_menu_tree_view: Popover = builder.get_object("gtk_context_menu_tree_view").expect("Couldn't get gtk_context_menu_tree_view");

    let top_menu_file_new_packfile: MenuItem = builder.get_object("gtk_top_menu_file_new_packfile").expect("Couldn't get gtk_top_menu_file_new_packfile");
    let top_menu_file_open_packfile: MenuItem = builder.get_object("gtk_top_menu_file_open_packfile").expect("Couldn't get gtk_top_menu_file_open_packfile");
    let top_menu_file_save_packfile: MenuItem = builder.get_object("gtk_top_menu_file_save_packfile").expect("Couldn't get gtk_top_menu_file_save_packfile");
    let top_menu_file_save_packfile_as: MenuItem = builder.get_object("gtk_top_menu_file_save_packfile_as").expect("Couldn't get gtk_top_menu_file_save_packfile_as");
    let top_menu_file_quit: MenuItem = builder.get_object("gtk_top_menu_file_quit").expect("Couldn't get gtk_top_menu_file_quit");
    let top_menu_special_patch_ai: MenuItem = builder.get_object("gtk_top_menu_special_patch_ai").expect("Couldn't get gtk_top_menu_special_patch_ai");
    let top_menu_about_about: MenuItem = builder.get_object("gtk_top_menu_about_about").expect("Couldn't get gtk_top_menu_about_about");

    let top_menu_file_change_packfile_type: MenuItem = builder.get_object("gtk_top_menu_file_select_packfile_type").expect("Couldn't get gtk_top_menu_file_select_packfile_type");
    let top_menu_file_change_packfile_type_boot: CheckMenuItem = builder.get_object("gtk_top_menu_file_select_packfile_type1").expect("Couldn't get gtk_top_menu_file_select_packfile_type1");
    let top_menu_file_change_packfile_type_release: CheckMenuItem = builder.get_object("gtk_top_menu_file_select_packfile_type2").expect("Couldn't get gtk_top_menu_file_select_packfile_type2");
    let top_menu_file_change_packfile_type_patch: CheckMenuItem = builder.get_object("gtk_top_menu_file_select_packfile_type3").expect("Couldn't get gtk_top_menu_file_select_packfile_type3");
    let top_menu_file_change_packfile_type_mod: CheckMenuItem = builder.get_object("gtk_top_menu_file_select_packfile_type4").expect("Couldn't get gtk_top_menu_file_select_packfile_type4");
    let top_menu_file_change_packfile_type_movie: CheckMenuItem = builder.get_object("gtk_top_menu_file_select_packfile_type5").expect("Couldn't get gtk_top_menu_file_select_packfile_type5");

    let folder_tree_view: TreeView = builder.get_object("gtk_folder_tree_view").expect("Couldn't get gtk_folder_tree_view");
    let folder_tree_selection: TreeSelection = builder.get_object("gtk_folder_tree_view_selection").expect("Couldn't get gtk_folder_tree_view_selection");

    // The TreeView's stuff is created manually here, as I had problems creating it in Glade.
    let folder_tree_store = TreeStore::new(&[String::static_type()]);
    folder_tree_view.set_model(Some(&folder_tree_store));

    let column = TreeViewColumn::new();
    let cell = CellRendererText::new();
    column.pack_start(&cell, true);
    column.add_attribute(&cell, "text", 0);

    folder_tree_view.append_column(&column);
    folder_tree_view.set_enable_tree_lines(true);
    folder_tree_view.set_enable_search(false);
    folder_tree_view.set_rules_hint(true);
    window.set_position(WindowPosition::Center);

    // We bring up the main window.
    window.show_all();

    // We also create a dummy PackFile we're going to use to store all the data from the opened Packfile.
    let pack_file_decoded = Rc::new(RefCell::new(pack_file_manager::pack_file::PackFile::new()));

    // End of the "Getting Ready" part.
    // From here, it's all event handling.

    // First, we catch the close window event, and close the program when we do it.
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    /*
    --------------------------------------------------------
                     Superior Menu: "File"
    --------------------------------------------------------
    */

    // When we open the menu, we check if we need to enable or disable his buttons first.
    let top_menu_file_save_packfile_toggle = top_menu_file_save_packfile.clone();
    let top_menu_file_save_packfile_as_toggle = top_menu_file_save_packfile_as.clone();
    let top_menu_file_change_packfile_type_toggle = top_menu_file_change_packfile_type.clone();
    let pack_file_decoded_toggle = pack_file_decoded.clone();

    top_menu_file.connect_activate(move |_| {

        // If the current PackFile has no name, we haven't open or created one, so disable all the
        // options that need a PackFile opened. Otherwise enable them.
        if pack_file_decoded_toggle.borrow_mut().pack_file_extra_data.file_name.is_empty() {
            top_menu_file_save_packfile_toggle.set_sensitive(false);
            top_menu_file_save_packfile_as_toggle.set_sensitive(false);
            top_menu_file_change_packfile_type_toggle.set_sensitive(false);
        }
        else {
            top_menu_file_save_packfile_toggle.set_sensitive(true);
            top_menu_file_save_packfile_as_toggle.set_sensitive(true);
            top_menu_file_change_packfile_type_toggle.set_sensitive(true);
        }
    });


    // When we hit the "New PackFile" button.
    let pack_file_decoded_new_packfile = pack_file_decoded.clone();
    let folder_tree_store_new_packfile = folder_tree_store.clone();
    let top_menu_file_change_packfile_type_mod_new_packfile = top_menu_file_change_packfile_type_mod.clone();

    top_menu_file_new_packfile.connect_activate(move |_| {

        // We just create a new PackFile with a name, set his type to Mod and update the
        // TreeView to show it.
        *pack_file_decoded_new_packfile.borrow_mut() = pack_file_manager::new_packfile("Unkown.pack".to_string());
        ui::update_tree_view(&folder_tree_store_new_packfile, &mut *pack_file_decoded_new_packfile.borrow_mut());
        top_menu_file_change_packfile_type_mod_new_packfile.set_active(true);
    });


    // When we hit the "Open PackFile" button.
    let pack_file_decoded_open_packfile = pack_file_decoded.clone();
    let folder_tree_store_open_packfile = folder_tree_store.clone();
    let error_dialog_open_packfile = error_dialog.clone();
    let top_menu_file_change_packfile_type_boot_open_packfile = top_menu_file_change_packfile_type_boot.clone();
    let top_menu_file_change_packfile_type_release_open_packfile = top_menu_file_change_packfile_type_release.clone();
    let top_menu_file_change_packfile_type_patch_open_packfile = top_menu_file_change_packfile_type_patch.clone();
    let top_menu_file_change_packfile_type_mod_open_packfile = top_menu_file_change_packfile_type_mod.clone();
    let top_menu_file_change_packfile_type_movie_open_packfile = top_menu_file_change_packfile_type_movie.clone();

    top_menu_file_open_packfile.connect_activate(move |_| {

        // When we select the file to open, we get his path, open it and, if there has been no
        // errors, decode it, update the TreeView to show it and check his type for the Change FilePack
        // Type option in the File menu.
        if file_chooser_open_packfile_dialog.run() == gtk::ResponseType::Ok.into() {
            let pack_file_path = file_chooser_open_packfile_dialog.get_filename().expect("Couldn't open file");
            match pack_file_manager::open_packfile(pack_file_path) {
                Ok(pack_file_decoded) => {

                    *pack_file_decoded_open_packfile.borrow_mut() = pack_file_decoded;
                    ui::update_tree_view(&folder_tree_store_open_packfile, &mut *pack_file_decoded_open_packfile.borrow_mut());

                    // Seleccionamos la opcion correcta del menu de tipo de packfile
                    if pack_file_decoded_open_packfile.borrow_mut().pack_file_header.pack_file_type == 0u32 {
                        top_menu_file_change_packfile_type_boot_open_packfile.set_active(true);
                    }
                    else if pack_file_decoded_open_packfile.borrow_mut().pack_file_header.pack_file_type == 1u32{
                        top_menu_file_change_packfile_type_release_open_packfile.set_active(true);
                    }
                    else if pack_file_decoded_open_packfile.borrow_mut().pack_file_header.pack_file_type == 2u32{
                        top_menu_file_change_packfile_type_patch_open_packfile.set_active(true);
                    }
                    else if pack_file_decoded_open_packfile.borrow_mut().pack_file_header.pack_file_type == 3u32{
                        top_menu_file_change_packfile_type_mod_open_packfile.set_active(true);
                    }
                    else if pack_file_decoded_open_packfile.borrow_mut().pack_file_header.pack_file_type == 4u32{
                        top_menu_file_change_packfile_type_movie_open_packfile.set_active(true);
                    }
                }
                Err(e) => {
                    ui::show_dialog(&error_dialog_open_packfile, e);
                }
            }
        }
        file_chooser_open_packfile_dialog.hide_on_delete();
    });


    // When we hit the "Save PackFile" button
    let error_dialog_save_packfile = error_dialog.clone();
    let success_dialog_save_packfile = success_dialog.clone();
    let folder_tree_view_save_packfile = folder_tree_view.clone();
    let folder_tree_selection_save_packfile = folder_tree_selection.clone();
    let pack_file_decoded_save_packfile = pack_file_decoded.clone();
    let folder_tree_store_save_packfile = folder_tree_store.clone();
    let file_chooser_save_packfile_dialog_normal = file_chooser_save_packfile_dialog.clone();

    top_menu_file_save_packfile.connect_activate(move |_| {

        // First, we check if our PackFile has a path. If it doesn't have it, we launch the Save
        // Dialog and set the current name in the extry of the dialog to his name.
        // When we hit "Accept", we get the selected path, encode the PackFile, and save it to that
        // path. After that, we update the TreeView to reflect the name change and hide the dialog.
        let mut pack_file_path: Option<PathBuf> = None;
        if pack_file_decoded_save_packfile.borrow_mut().pack_file_extra_data.file_path.is_empty() {
            file_chooser_save_packfile_dialog_normal.set_current_name(&pack_file_decoded_save_packfile.borrow_mut().pack_file_extra_data.file_name);
            if file_chooser_save_packfile_dialog_normal.run() == gtk::ResponseType::Ok.into() {
                pack_file_path = Some(file_chooser_save_packfile_dialog_normal.get_filename().expect("Couldn't open file"));
                match pack_file_manager::save_packfile( &mut *pack_file_decoded_save_packfile.borrow_mut(), pack_file_path) {
                    Ok(result) => {
                        ui::show_dialog(&success_dialog_save_packfile, result)
                    }
                    Err(result) => {
                        ui::show_dialog(&error_dialog_save_packfile, result)
                    }
                }

                ui::update_tree_view_expand_path(
                    &folder_tree_store_save_packfile,
                    &mut *pack_file_decoded_save_packfile.borrow_mut(),
                    &folder_tree_selection_save_packfile,
                    &folder_tree_view_save_packfile,
                    false
                );
            }
            file_chooser_save_packfile_dialog_normal.hide_on_delete();
        }

        // If the PackFile has a path, we just encode it and save it into that path.
        else {
            match pack_file_manager::save_packfile( &mut *pack_file_decoded_save_packfile.borrow_mut(), pack_file_path) {
                Ok(result) => {
                    ui::show_dialog(&success_dialog_save_packfile, result)
                }
                Err(result) => {
                    ui::show_dialog(&error_dialog_save_packfile, result)
                }
            }
        }
    });


    // When we hit the "Save PackFile as" button.
    let error_dialog_save_packfile_as = error_dialog.clone();
    let success_dialog_save_packfile_as = success_dialog.clone();
    let folder_tree_store_save_packfile_as = folder_tree_store.clone();
    let folder_tree_view_save_packfile_as = folder_tree_view.clone();
    let folder_tree_selection_save_packfile_as = folder_tree_selection.clone();
    let pack_file_decoded_save_packfile_as = pack_file_decoded.clone();
    let file_chooser_save_packfile_dialog_as = file_chooser_save_packfile_dialog.clone();

    top_menu_file_save_packfile_as.connect_activate(move |_| {

        // We first set the current file of the Save dialog to the PackFile's name. Then we just
        // encode it and save it in the path selected. After that, we update the TreeView to reflect
        // the name change and hide the dialog.
        file_chooser_save_packfile_dialog_as.set_current_name(&pack_file_decoded_save_packfile_as.borrow_mut().pack_file_extra_data.file_name);
        if file_chooser_save_packfile_dialog_as.run() == gtk::ResponseType::Ok.into() {
            match pack_file_manager::save_packfile(
                &mut *pack_file_decoded_save_packfile_as.borrow_mut(),
               Some(file_chooser_save_packfile_dialog_as.get_filename().expect("Couldn't open file"))) {
                Ok(result) => {
                    ui::show_dialog(&success_dialog_save_packfile_as, result);
                }
                Err(result) => {
                    ui::show_dialog(&error_dialog_save_packfile_as, result)
                }
            }

            ui::update_tree_view_expand_path(
                &folder_tree_store_save_packfile_as,
                &mut *pack_file_decoded_save_packfile_as.borrow_mut(),
                &folder_tree_selection_save_packfile_as,
                &folder_tree_view_save_packfile_as,
                false
            );
        }
        file_chooser_save_packfile_dialog.hide_on_delete();
    });


    // When changing the type of the PackFile... we just change his pack_file_type variable. Nothing complex.
    let top_menu_file_change_packfile_type_boot_change_packfile_type = top_menu_file_change_packfile_type_boot.clone();
    let top_menu_file_change_packfile_type_release_change_packfile_type = top_menu_file_change_packfile_type_release.clone();
    let top_menu_file_change_packfile_type_patch_change_packfile_type = top_menu_file_change_packfile_type_patch.clone();
    let top_menu_file_change_packfile_type_mod_change_packfile_type = top_menu_file_change_packfile_type_mod.clone();
    let top_menu_file_change_packfile_type_movie_change_packfile_type = top_menu_file_change_packfile_type_movie.clone();
    let pack_file_decoded_change_packfile_type1 = pack_file_decoded.clone();
    let pack_file_decoded_change_packfile_type2 = pack_file_decoded.clone();
    let pack_file_decoded_change_packfile_type3 = pack_file_decoded.clone();
    let pack_file_decoded_change_packfile_type4 = pack_file_decoded.clone();
    let pack_file_decoded_change_packfile_type5 = pack_file_decoded.clone();

    top_menu_file_change_packfile_type_boot.connect_toggled(move |_| {
        if top_menu_file_change_packfile_type_boot_change_packfile_type.get_active() {
            pack_file_decoded_change_packfile_type1.borrow_mut().pack_file_header.pack_file_type = 0;
        }
    });
    top_menu_file_change_packfile_type_release.connect_toggled(move |_| {
        if top_menu_file_change_packfile_type_release_change_packfile_type.get_active() {
            pack_file_decoded_change_packfile_type2.borrow_mut().pack_file_header.pack_file_type = 1;
        }
    });
    top_menu_file_change_packfile_type_patch.connect_toggled(move |_| {
        if top_menu_file_change_packfile_type_patch_change_packfile_type.get_active() {
            pack_file_decoded_change_packfile_type3.borrow_mut().pack_file_header.pack_file_type = 2;
        }
    });
    top_menu_file_change_packfile_type_mod.connect_toggled(move |_| {
        if top_menu_file_change_packfile_type_mod_change_packfile_type.get_active() {
            pack_file_decoded_change_packfile_type4.borrow_mut().pack_file_header.pack_file_type = 3;
        }
    });
    top_menu_file_change_packfile_type_movie.connect_toggled(move |_| {
        if top_menu_file_change_packfile_type_movie_change_packfile_type.get_active() {
            pack_file_decoded_change_packfile_type5.borrow_mut().pack_file_header.pack_file_type = 4;
        }
    });


    // When we hit the "Quit" button.
    top_menu_file_quit.connect_activate(|_| {
        gtk::main_quit();
    });

    /*
    --------------------------------------------------------
                 Superior Menu: "Special Stuff"
    --------------------------------------------------------
    */

    // When we open the menu, we check if we need to enable or disable his buttons first.
    let top_menu_special_patch_ai_toggle = top_menu_special_patch_ai.clone();
    let pack_file_decoded_toggle = pack_file_decoded.clone();

    top_menu_special_stuff.connect_activate(move |_| {
        if pack_file_decoded_toggle.borrow_mut().pack_file_extra_data.file_name.is_empty() {
            top_menu_special_patch_ai_toggle.set_sensitive(false);
        }
        else {
            top_menu_special_patch_ai_toggle.set_sensitive(true);
        }
    });


    // When we hit the "Patch SiegeAI" button.
    let pack_file_decoded_patch_ai = pack_file_decoded.clone();
    let folder_tree_store_patch_ai = folder_tree_store.clone();
    let error_dialog_patch_ai = error_dialog.clone();
    let success_dialog_patch_ai = success_dialog.clone();
    let folder_tree_view_patch_ai = folder_tree_view.clone();
    let folder_tree_selection_patch_ai = folder_tree_selection.clone();

    top_menu_special_patch_ai.connect_activate(move |_| {

        // First, we try to patch the PackFile. If there are no errors, we save the result in a tuple.
        // Then we check that tuple and, if it's a success, we save the PackFile and update the TreeView.
        let mut sucessful_patching = (false, String::new());
        match pack_file_manager::patch_siege_ai(&mut *pack_file_decoded_patch_ai.borrow_mut()) {
            Ok(result) => {
                sucessful_patching = (true, result);
            }
            Err(result) => {
                ui::show_dialog(&error_dialog_patch_ai, result)
            }
        }
        if sucessful_patching.0 {
            match pack_file_manager::save_packfile( &mut *pack_file_decoded_patch_ai.borrow_mut(), None) {
                Ok(result) => {
                    ui::show_dialog(&success_dialog_patch_ai, format!("{}\n\n{}", sucessful_patching.1, result));
                }
                Err(_) => {
                    ui::show_dialog(&error_dialog_patch_ai, sucessful_patching.1)
                }
            }
            ui::update_tree_view_expand_path(
                &folder_tree_store_patch_ai,
                &mut *pack_file_decoded_patch_ai.borrow_mut(),
                &folder_tree_selection_patch_ai,
                &folder_tree_view_patch_ai,
                false
            );
        }
    });

    /*
    --------------------------------------------------------
                    Superior Menu: "About"
    --------------------------------------------------------
    */

    // When we hit the "About" button.
    top_menu_about_about.connect_activate(move |_| {
        window_about.run();
        window_about.hide_on_delete();
    });


    /*
    --------------------------------------------------------
                   Contextual TreeView Popup
    --------------------------------------------------------
    */

    // When we right-click the TreeView, we check if we need to enable or disable his buttons first.
    // Then we calculate the position where the popup must aim, and show it.
    //
    // NOTE: REMEMBER, WE OPEN THE POPUP HERE, BUT WE NEED TO CLOSED IT WHEN WE HIT HIS BUTTONS.
    let pack_file_decoded_context_toggle = pack_file_decoded.clone();
    let tree_view_add_file_context_toggle = tree_view_add_file.clone();
    let tree_view_add_folder_context_toggle = tree_view_add_folder.clone();
    let tree_view_extract_file_context_toggle = tree_view_extract_file.clone();
    let folder_tree_selection_context_toggle = folder_tree_selection.clone();
    let folder_tree_view_context_toggle = folder_tree_view.clone();
    let context_menu_tree_view_popup = context_menu_tree_view.clone();

    folder_tree_view.connect_button_release_event(move |_, button,| {

        let button_val = button.get_button();
        if button_val == 3 && folder_tree_selection_context_toggle.count_selected_rows() > 0 {
            let tree_path = ui::get_tree_path_from_selection(&folder_tree_selection_context_toggle, false);
            for i in &*pack_file_decoded_context_toggle.borrow_mut().pack_file_data.packed_files {
                // If the selected thing is a file
                if i.packed_file_path == tree_path {
                    tree_view_add_file_context_toggle.set_sensitive(false);
                    tree_view_add_folder_context_toggle.set_sensitive(false);
                    tree_view_extract_file_context_toggle.set_sensitive(true);
                    break;
                }
                else {
                    tree_view_add_file_context_toggle.set_sensitive(true);
                    tree_view_add_folder_context_toggle.set_sensitive(true);
                    tree_view_extract_file_context_toggle.set_sensitive(true);
                }
            }
            if tree_path.len() == 0 {
                tree_view_extract_file_context_toggle.set_sensitive(false);
            }
            let rect = ui::get_rect_for_popover(&folder_tree_selection_context_toggle, &folder_tree_view_context_toggle);

            context_menu_tree_view_popup.set_pointing_to(&rect);
            context_menu_tree_view_popup.popup();
        }
        Inhibit(false)
    });


    // When we hit the "Add file" button.
    let pack_file_decoded_add_file_to_packfile = pack_file_decoded.clone();
    let folder_tree_store_add_file_to_packfile = folder_tree_store.clone();
    let folder_tree_selection_add_file_to_packfile = folder_tree_selection.clone();
    let folder_tree_view_add_file_to_packfile = folder_tree_view.clone();
    let context_menu_tree_view_popdown = context_menu_tree_view.clone();
    let error_dialog_add_file_to_packfile = error_dialog.clone();

    tree_view_add_file.connect_button_release_event(move |_,_| {

        // First, we hide the context menu, then we pick the file selected and add it to the Packfile.
        // After that, we update the TreeView.
        context_menu_tree_view_popdown.popdown();

        if file_chooser_add_file_to_packfile.run() == gtk::ResponseType::Ok.into() {
            let file_path = file_chooser_add_file_to_packfile.get_filename().expect("Couldn't open file");
            let tree_path = ui::get_tree_path_from_pathbuf(&file_path, &folder_tree_selection_add_file_to_packfile, true);
            let mut file_added = false;
            match pack_file_manager::add_file_to_packfile(&mut *pack_file_decoded_add_file_to_packfile.borrow_mut(), file_path, tree_path) {
                Ok(_) => {
                    file_added = true;
                }
                Err(result) => {
                    ui::show_dialog(&error_dialog_add_file_to_packfile, result);
                }
            }
            if file_added {
                ui::update_tree_view_expand_path(
                    &folder_tree_store_add_file_to_packfile,
                    &mut *pack_file_decoded_add_file_to_packfile.borrow_mut(),
                    &folder_tree_selection_add_file_to_packfile,
                    &folder_tree_view_add_file_to_packfile,
                    false
                );
            }
        }
        file_chooser_add_file_to_packfile.hide_on_delete();

        Inhibit(false)
    });


    // When we hit the "Add folder" button.
    let pack_file_decoded_add_folder_to_packfile = pack_file_decoded.clone();
    let folder_tree_store_add_folder_to_packfile = folder_tree_store.clone();
    let folder_tree_selection_add_folder_to_packfile = folder_tree_selection.clone();
    let folder_tree_view_add_folder_to_packfile = folder_tree_view.clone();
    let context_menu_tree_view_popdown = context_menu_tree_view.clone();
    let error_dialog_add_folder_to_packfile = error_dialog.clone();

    tree_view_add_folder.connect_button_release_event(move |_,_| {

        // First, we hide the context menu. Then we get the folder selected and we get all the files
        // in him and his subfolders. After that, for every one of those files, we strip his path,
        // leaving then with only the part that will be added to the PackedFile and we add it to the
        // PackFile. After all that, if we added any of the files to the PackFile, we update the
        // TreeView.
        context_menu_tree_view_popdown.popdown();
        if file_chooser_add_folder_to_packfile.run() == gtk::ResponseType::Ok.into() {
            let big_parent = file_chooser_add_folder_to_packfile.get_filename().unwrap();
            let mut big_parent_prefix = big_parent.clone();
            big_parent_prefix.pop();
            let file_path_list = ::common::get_files_from_subdir(&big_parent);
            let mut file_errors = 0;
            for i in file_path_list {
                match i.strip_prefix(&big_parent_prefix) {
                    Ok(filtered_path) => {
                        let tree_path = ui::get_tree_path_from_pathbuf(&filtered_path.to_path_buf(), &folder_tree_selection_add_folder_to_packfile, false);
                        match pack_file_manager::add_file_to_packfile(&mut *pack_file_decoded_add_folder_to_packfile.borrow_mut(), i.to_path_buf(), tree_path) {
                            Ok(_) => {

                            }
                            Err(_) => {
                                file_errors += 1;
                            }
                        }
                    }
                    Err(_) => {
                        panic!("Error while trying to filter the path. This should never happend unless I break something while I'm getting the paths.");
                    }
                }
            }
            if file_errors > 0 {
                ui::show_dialog(&error_dialog_add_folder_to_packfile, format!("{} file/s that you wanted to add already exist in the Packfile.", file_errors));
            }
            ui::update_tree_view_expand_path(
                &folder_tree_store_add_folder_to_packfile,
                &mut *pack_file_decoded_add_folder_to_packfile.borrow_mut(),
                &folder_tree_selection_add_folder_to_packfile,
                &folder_tree_view_add_folder_to_packfile,
                false
            );
        }
        file_chooser_add_folder_to_packfile.hide_on_delete();

        Inhibit(false)
    });


    // When we hit the "Delete file/folder" button.
    let pack_file_decoded_delete_file_from_packfile = pack_file_decoded.clone();
    let folder_tree_store_delete_file_from_packfile = folder_tree_store.clone();
    let folder_tree_selection_delete_file_from_packfile = folder_tree_selection.clone();
    let folder_tree_view_delete_file_from_packfile = folder_tree_view.clone();
    let context_menu_tree_view_popdown = context_menu_tree_view.clone();

    tree_view_delete_file.connect_button_release_event(move |_,_,|{

        // We hide the context menu, then we get the selected file/folder, delete it and update the
        // TreeView. Pretty simple, actually.
        context_menu_tree_view_popdown.popdown();

        let tree_path = ui::get_tree_path_from_selection(&folder_tree_selection_delete_file_from_packfile, false);
        pack_file_manager::delete_from_packfile(&mut *pack_file_decoded_delete_file_from_packfile.borrow_mut(), tree_path);
        ui::update_tree_view_expand_path(
            &folder_tree_store_delete_file_from_packfile,
            &mut *pack_file_decoded_delete_file_from_packfile.borrow_mut(),
            &folder_tree_selection_delete_file_from_packfile,
            &folder_tree_view_delete_file_from_packfile,
            true
        );
        Inhibit(false)
    });


    // When we hit the "Extract file/folder" button.
    let pack_file_decoded_extract_from_packfile = pack_file_decoded.clone();
    let folder_tree_selection_extract_from_packfile = folder_tree_selection.clone();
    let error_dialog_extract_from_packfile = error_dialog.clone();
    let success_dialog_extract_from_packfile = success_dialog.clone();
    let context_menu_tree_view_popdown = context_menu_tree_view.clone();

    tree_view_extract_file.connect_button_release_event(move |_,_,|{

        // First, we hide the context menu.
        context_menu_tree_view_popdown.popdown();

        let tree_path = ui::get_tree_path_from_selection(&folder_tree_selection_extract_from_packfile, false);

        // Then, we check with the correlation data if the tree_path is a folder or a file.
        let mut is_a_file = false;
        for i in &*pack_file_decoded_extract_from_packfile.borrow().pack_file_extra_data.correlation_data {
            if i == &tree_path {
                is_a_file = true;
                break;
            }
        }

        // Both (folder and file) are processed in the same way but we need a different
        // FileChooser for files and folders, so we check first what it's.
        if is_a_file {
            file_chooser_extract_file.set_current_name(&tree_path.last().unwrap());
            if file_chooser_extract_file.run() == gtk::ResponseType::Ok.into() {
                match pack_file_manager::extract_from_packfile(
                    &*pack_file_decoded_extract_from_packfile.borrow_mut(),
                    tree_path,
                    file_chooser_extract_file.get_filename().expect("Couldn't open file")) {
                    Ok(result) => {
                        ui::show_dialog(&success_dialog_extract_from_packfile, result);
                    }
                    Err(result) => {
                        ui::show_dialog(&error_dialog_extract_from_packfile, result)
                    }
                }
            }
            file_chooser_extract_file.hide_on_delete();
        }
        else {
            if file_chooser_extract_folder.run() == gtk::ResponseType::Ok.into() {
                match pack_file_manager::extract_from_packfile(
                    &*pack_file_decoded_extract_from_packfile.borrow_mut(),
                    tree_path,
                    file_chooser_extract_folder.get_filename().expect("Couldn't open file")) {
                    Ok(result) => {
                        ui::show_dialog(&success_dialog_extract_from_packfile, result);
                    }
                    Err(result) => {
                        ui::show_dialog(&error_dialog_extract_from_packfile, result)
                    }
                }
            }
            file_chooser_extract_folder.hide_on_delete();
        }

        Inhibit(false)
    });

    /*
    --------------------------------------------------------
                        Special Events
    --------------------------------------------------------
    */

    // When we double-click something in the TreeView (or click something already selected).
    let pack_file_decoded_rename_packed_file = pack_file_decoded.clone();
    let folder_tree_store_rename_packed_file = folder_tree_store.clone();
    let folder_tree_selection_rename_packed_file = folder_tree_selection.clone();
    let folder_tree_view_rename_packed_file = folder_tree_view.clone();
    let rename_popover_rename_packed_file = rename_popover.clone();
    let rename_popover_text_entry_rename_packed_file = rename_popover_text_entry.clone();
    let error_dialog_rename_packed_file = error_dialog.clone();

    folder_tree_view.connect_row_activated(move |_,_,_,| {

        // First, we re-clone the variables to be able to use them in the second closure.
        let pack_file_decoded_rename_packed_file2 = pack_file_decoded_rename_packed_file.clone();
        let folder_tree_store_rename_packed_file2 = folder_tree_store_rename_packed_file.clone();
        let rename_popover_rename_packed_file2 = rename_popover_rename_packed_file.clone();
        let rename_popover_text_entry_rename_packed_file2 = rename_popover_text_entry_rename_packed_file.clone();
        let error_dialog_rename_packed_file2 = error_dialog_rename_packed_file.clone();
        let folder_tree_selection_rename_packed_file2 = folder_tree_selection_rename_packed_file.clone();
        let folder_tree_view_rename_packed_file2 = folder_tree_view_rename_packed_file.clone();
        let new_name: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
        let new_name_cloned = new_name.clone();

        let rect = ui::get_rect_for_popover(&folder_tree_selection_rename_packed_file, &folder_tree_view_rename_packed_file);
        rename_popover_rename_packed_file.set_pointing_to(&rect);
        rename_popover_rename_packed_file.popup();

        // Now, in the "New Name" popup, we wait until "Enter" (65293) is hit AND released.
        // In that point, we try to rename the file/folder selected. If we success, the TreeView is
        // updated. If not, we get a Dialog saying why.
        rename_popover_rename_packed_file.connect_key_release_event(move |_, key,| {
            let key_val = key.get_keyval();
            if key_val == 65293 {
                let mut name_changed = false;
                let tree_path = ui::get_tree_path_from_selection(&folder_tree_selection_rename_packed_file2, true);
                *new_name_cloned.borrow_mut() = rename_popover_text_entry_rename_packed_file2.get_buffer().get_text();
                match pack_file_manager::rename_packed_file(&mut *pack_file_decoded_rename_packed_file2.borrow_mut(), tree_path.to_vec(), &*new_name.borrow()) {
                    Ok(_) => {
                        rename_popover_rename_packed_file2.popdown();
                        name_changed = true;
                    }
                    Err(result) => {
                        ui::show_dialog(&error_dialog_rename_packed_file2, result);
                    }
                }
                if name_changed {
                    ui::update_tree_view_expand_path(
                        &folder_tree_store_rename_packed_file2,
                        &mut *pack_file_decoded_rename_packed_file2.borrow_mut(),
                        &folder_tree_selection_rename_packed_file2,
                        &folder_tree_view_rename_packed_file2,
                        true
                    );
                }
                rename_popover_text_entry_rename_packed_file2.get_buffer().set_text("");
            }
            // We need to set this to true to avoid the Enter re-fire this event again and again.
            Inhibit(true)
        });
        Inhibit(true);
    });


    // We start GTK. Yay
    gtk::main();
}



