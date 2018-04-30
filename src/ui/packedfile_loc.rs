// In this file are all the helper functions used by the UI when decoding Loc PackedFiles.
extern crate gtk;
extern crate gio;
extern crate failure;

use std::cell::RefCell;
use std::rc::Rc;
use failure::Error;
use gio::prelude::*;
use gio::SimpleAction;
use gtk::prelude::*;
use gtk::{
    TreeView, ListStore, ScrolledWindow, Popover, Entry, ModelButton, FileChooserAction,
    CellRendererText, TreeViewColumn, CellRendererToggle, Separator, Orientation, Grid,
    TreeViewColumnSizing, TreeViewGridLines, EntryIconPosition, Application
};

use packedfile::loc::*;
use packfile::update_packed_file_data_loc;
use settings::*;
use ui::*;
use AppUI;
use packedfile::SerializableToTSV;

/// Struct `PackedFileLocTreeView`: contains all the stuff we need to give to the program to show a
/// `TreeView` with the data of a Loc PackedFile, allowing us to manipulate it.
#[derive(Clone)]
pub struct PackedFileLocTreeView {
    pub tree_view: TreeView,
    pub list_store: ListStore,
    pub cell_key: CellRendererText,
    pub cell_text: CellRendererText,
    pub cell_tooltip: CellRendererToggle,
    pub context_menu: Popover,
    pub add_rows_entry: Entry,
}

/// Implementation of `PackedFileLocTreeView`.
impl PackedFileLocTreeView{

    /// This function creates a new `TreeView` with `packed_file_data_display` as father and returns a
    /// `PackedFileLocTreeView` with all his data.
    pub fn create_tree_view(
        application: &Application,
        app_ui: &AppUI,
        pack_file: Rc<RefCell<PackFile>>,
        packed_file_decoded: Rc<RefCell<Loc>>,
        packed_file_decoded_index: &usize,
        settings: &Settings
    ) {

        // Here we define the `Accept` response for GTK, as it seems Restson causes it to fail to compile
        // if we get them to i32 directly in the `if` statement.
        // NOTE: For some bizarre reason, GTKFileChoosers return `Ok`, while native ones return `Accept`.
        let gtk_response_accept: i32 = ResponseType::Accept.into();

        // Get the table's path, so we can use it despite changing the selected file in the main TreeView.
        let tree_path = get_tree_path_from_selection(&app_ui.folder_tree_selection, false);

        // We create the new `TreeView` and his `ListStore`.
        let tree_view = TreeView::new();
        let list_store = ListStore::new(&[String::static_type(), String::static_type(), String::static_type(), gtk::Type::Bool]);

        // Config the `TreeView`.
        tree_view.set_model(Some(&list_store));
        tree_view.set_grid_lines(TreeViewGridLines::Both);
        tree_view.set_rubber_banding(true);
        tree_view.set_enable_search(false);
        tree_view.set_search_column(1);
        tree_view.set_margin_bottom(10);

        // We enable "Multiple" selection mode, so we can do multi-row operations.
        tree_view.get_selection().set_mode(gtk::SelectionMode::Multiple);

        // Create the four type of cells we are going to use.
        let cell_index = CellRendererText::new();
        let cell_key = CellRendererText::new();
        let cell_text = CellRendererText::new();
        let cell_tooltip = CellRendererToggle::new();

        // Config the cells.
        cell_key.set_property_editable(true);
        cell_text.set_property_editable(true);
        cell_tooltip.set_activatable(true);

        // Reduce the size of the checkbox to 160% the size of the font used (yes, 160% is less that his normal size).
        cell_tooltip.set_property_indicator_size((settings.font.split(' ').filter_map(|x| x.parse::<f32>().ok()).collect::<Vec<f32>>()[0] * 1.6) as i32);

        // Create the four columns we are going to use.
        let column_index = TreeViewColumn::new();
        let column_key = TreeViewColumn::new();
        let column_text = TreeViewColumn::new();
        let column_tooltip = TreeViewColumn::new();

        // Set the column's titles.
        column_index.set_title("Index");
        column_key.set_title("Key");
        column_text.set_title("Text");
        column_tooltip.set_title("Tooltip");

        // Make the headers clickable.
        column_index.set_clickable(true);
        column_key.set_clickable(true);
        column_text.set_clickable(true);
        column_tooltip.set_clickable(true);

        // Allow the user to move the columns. Because why not?
        column_key.set_reorderable(true);
        column_text.set_reorderable(true);
        column_tooltip.set_reorderable(true);

        // Allow the user to resize the "key" and "text" columns.
        column_key.set_resizable(true);
        column_text.set_resizable(true);

        // Set a minimal width for the columns, so they can't be fully hidden.
        column_index.set_max_width(60);
        column_key.set_min_width(50);
        column_text.set_min_width(50);
        column_tooltip.set_min_width(50);

        // Make both "key" and "text" columns be able to grow from his minimum size.
        column_key.set_sizing(TreeViewColumnSizing::GrowOnly);
        column_text.set_sizing(TreeViewColumnSizing::GrowOnly);

        // Center the column's titles.
        column_index.set_alignment(0.5);
        column_key.set_alignment(0.5);
        column_text.set_alignment(0.5);
        column_tooltip.set_alignment(0.5);

        // Set the ID's to short the columns.
        column_index.set_sort_column_id(0);
        column_key.set_sort_column_id(1);
        column_text.set_sort_column_id(2);
        column_tooltip.set_sort_column_id(3);

        // Add the cells to the columns.
        column_index.pack_start(&cell_index, true);
        column_key.pack_start(&cell_key, true);
        column_text.pack_start(&cell_text, true);
        column_tooltip.pack_start(&cell_tooltip, true);

        // Set their attributes, so we can manipulate their contents.
        column_index.add_attribute(&cell_index, "text", 0);
        column_key.add_attribute(&cell_key, "text", 1);
        column_text.add_attribute(&cell_text, "text", 2);
        column_tooltip.add_attribute(&cell_tooltip, "active", 3);

        // Add the four columns to the `TreeView`.
        tree_view.append_column(&column_index);
        tree_view.append_column(&column_key);
        tree_view.append_column(&column_text);
        tree_view.append_column(&column_tooltip);

        // Make an extra "Dummy" column that will expand to fill the space between the last column and
        // the right border of the window.
        let cell_dummy = CellRendererText::new();
        let column_dummy = TreeViewColumn::new();
        column_dummy.pack_start(&cell_dummy, true);
        tree_view.append_column(&column_dummy);

        // Here we create the Popover menu. It's created and destroyed with the table because otherwise
        // it'll start crashing when changing tables and trying to delete stuff. Stupid menu. Also, it can't
        // be created from a `MenuModel` like the rest, because `MenuModel`s can't hold an `Entry`.
        let context_menu = Popover::new(&tree_view);

        // Create the `Grid` that'll hold all the buttons in the Contextual Menu.
        let context_menu_grid = Grid::new();
        context_menu_grid.set_border_width(6);

        // Before setting up the actions, we clean the previous ones.
        remove_temporal_accelerators(&application);

        // Create the "Add row/s" button.
        let add_rows_button = ModelButton::new();
        add_rows_button.set_property_text(Some("Add row/s:"));
        add_rows_button.set_action_name("app.packedfile_loc_add_rows");

        // Create the entry to specify the amount of rows you want to add.
        let add_rows_entry = Entry::new();
        let add_rows_entry_buffer = add_rows_entry.get_buffer();
        add_rows_entry.set_alignment(1.0);
        add_rows_entry.set_width_chars(8);
        add_rows_entry.set_icon_from_icon_name(EntryIconPosition::Primary, Some("go-last"));
        add_rows_entry.set_has_frame(false);
        add_rows_entry_buffer.set_max_length(Some(4));
        add_rows_entry_buffer.set_text("1");

        // Create the "Delete row/s" button.
        let delete_rows_button = ModelButton::new();
        delete_rows_button.set_property_text(Some("Delete row/s"));
        delete_rows_button.set_action_name("app.packedfile_loc_delete_rows");

        // Create the separator between "Delete row/s" and the copy/paste buttons.
        let separator_1 = Separator::new(Orientation::Vertical);

        // Create the "Copy cell" button.
        let copy_cell_button = ModelButton::new();
        copy_cell_button.set_property_text(Some("Copy cell"));
        copy_cell_button.set_action_name("app.packedfile_loc_copy_cell");

        // Create the "Paste cell" button.
        let paste_cell_button = ModelButton::new();
        paste_cell_button.set_property_text(Some("Paste cell"));
        paste_cell_button.set_action_name("app.packedfile_loc_paste_cell");

        // Create the "Copy row/s" button.
        let copy_rows_button = ModelButton::new();
        copy_rows_button.set_property_text(Some("Copy row/s"));
        copy_rows_button.set_action_name("app.packedfile_loc_copy_rows");

        // Create the "Paste row/s" button.
        let paste_rows_button = ModelButton::new();
        paste_rows_button.set_property_text(Some("Paste row/s"));
        paste_rows_button.set_action_name("app.packedfile_loc_paste_rows");

        // Create the separator between the "Import/Export" buttons and the rest.
        let separator_2 = Separator::new(Orientation::Vertical);

        // Create the "Import from TSV" button.
        let import_tsv_button = ModelButton::new();
        import_tsv_button.set_property_text(Some("Import from TSV"));
        import_tsv_button.set_action_name("app.packedfile_loc_import_tsv");

        // Create the "Export to TSV" button.
        let export_tsv_button = ModelButton::new();
        export_tsv_button.set_property_text(Some("Export to TSV"));
        export_tsv_button.set_action_name("app.packedfile_loc_export_tsv");

        // Right-click menu actions.
        let add_rows = SimpleAction::new("packedfile_loc_add_rows", None);
        let delete_rows = SimpleAction::new("packedfile_loc_delete_rows", None);
        let copy_cell = SimpleAction::new("packedfile_loc_copy_cell", None);
        let paste_cell = SimpleAction::new("packedfile_loc_paste_cell", None);
        let copy_rows = SimpleAction::new("packedfile_loc_copy_rows", None);
        let paste_rows = SimpleAction::new("packedfile_loc_paste_rows", None);
        let import_tsv = SimpleAction::new("packedfile_loc_import_tsv", None);
        let export_tsv = SimpleAction::new("packedfile_loc_export_tsv", None);

        application.add_action(&add_rows);
        application.add_action(&delete_rows);
        application.add_action(&copy_cell);
        application.add_action(&paste_cell);
        application.add_action(&copy_rows);
        application.add_action(&paste_rows);
        application.add_action(&import_tsv);
        application.add_action(&export_tsv);

        // Accels for popovers need to be specified here. Don't know why, but otherwise they do not work.
        application.set_accels_for_action("app.packedfile_loc_add_rows", &["<Primary><Shift>a"]);
        application.set_accels_for_action("app.packedfile_loc_delete_rows", &["<Shift>Delete"]);
        application.set_accels_for_action("app.packedfile_loc_copy_cell", &["<Primary>c"]);
        application.set_accels_for_action("app.packedfile_loc_paste_cell", &["<Primary>v"]);
        application.set_accels_for_action("app.packedfile_loc_copy_rows", &["<Primary>z"]);
        application.set_accels_for_action("app.packedfile_loc_paste_rows", &["<Primary>x"]);
        application.set_accels_for_action("app.packedfile_loc_import_tsv", &["<Primary><Shift>i"]);
        application.set_accels_for_action("app.packedfile_loc_export_tsv", &["<Primary><Shift>e"]);

        // Some actions need to start disabled.
        delete_rows.set_enabled(false);
        copy_cell.set_enabled(false);
        copy_rows.set_enabled(false);
        paste_cell.set_enabled(false);

        // Depending of the current contents of the `Clipboard`, set the initial state of the "Paste rows" action.
        if app_ui.clipboard.wait_for_text().is_some() {

            // If the data in the clipboard is a valid row, we enable "Paste rows".
            if check_clipboard_row(&app_ui) { paste_rows.set_enabled(true); }

            // Otherwise, we disable the "Paste rows" action.
            else { paste_rows.set_enabled(false); }
        }
        else { paste_rows.set_enabled(false); }

        // Attach all the stuff to the Context Menu `Grid`.
        context_menu_grid.attach(&add_rows_button, 0, 0, 1, 1);
        context_menu_grid.attach(&add_rows_entry, 1, 0, 1, 1);
        context_menu_grid.attach(&delete_rows_button, 0, 1, 2, 1);
        context_menu_grid.attach(&separator_1, 0, 2, 2, 1);
        context_menu_grid.attach(&copy_cell_button, 0, 3, 2, 1);
        context_menu_grid.attach(&paste_cell_button, 0, 4, 2, 1);
        context_menu_grid.attach(&copy_rows_button, 0, 5, 2, 1);
        context_menu_grid.attach(&paste_rows_button, 0, 6, 2, 1);
        context_menu_grid.attach(&separator_2, 0, 7, 2, 1);
        context_menu_grid.attach(&import_tsv_button, 0, 8, 2, 1);
        context_menu_grid.attach(&export_tsv_button, 0, 9, 2, 1);

        // Add the `Grid` to the Context Menu and show it.
        context_menu.add(&context_menu_grid);
        context_menu.show_all();

        // Make a `ScrolledWindow` to put the `TreeView` into it.
        let packed_file_data_scroll = ScrolledWindow::new(None, None);
        packed_file_data_scroll.set_hexpand(true);
        packed_file_data_scroll.set_vexpand(true);

        // Add the `TreeView` to the `ScrolledWindow`, the `ScrolledWindow` to the main `Grid`, and show it.
        packed_file_data_scroll.add(&tree_view);
        app_ui.packed_file_data_display.attach(&packed_file_data_scroll, 0, 0, 1, 1);
        app_ui.packed_file_data_display.show_all();

        // Hide the Context Menu by default.
        context_menu.hide();

        // Return the struct with the Loc `TreeView` and all the stuff we want.
        let decoded_view = PackedFileLocTreeView {
            tree_view,
            list_store,
            cell_key,
            cell_text,
            cell_tooltip,
            context_menu,
            add_rows_entry,
        };

        // Then we populate the TreeView with the entries of the Loc PackedFile.
        PackedFileLocTreeView::load_data_to_tree_view(&packed_file_decoded.borrow().packed_file_data, &decoded_view.list_store);

        // Contextual Menu actions.
        {

            // When we right-click the `TreeView`, show the Contextual Menu.
            //
            // NOTE: REMEMBER, WE OPEN THE POPUP HERE, BUT WE NEED TO CLOSED IT WHEN WE HIT HIS BUTTONS.
            decoded_view.tree_view.connect_button_release_event(clone!(
                app_ui,
                paste_rows,
                decoded_view => move |tree_view, button| {

                    // If we clicked the right mouse button...
                    if button.get_button() == 3 {

                        // If we got text in the `Clipboard`...
                        if app_ui.clipboard.wait_for_text().is_some() {

                            // If the data in the clipboard is a valid row...
                            if check_clipboard_row(&app_ui) {

                                // We enable "Paste rows".
                                paste_rows.set_enabled(true);
                            }

                            // Otherwise, we disable the "Paste rows" action.
                            else { paste_rows.set_enabled(false); }
                        }
                        else { paste_rows.set_enabled(false); }

                        // Point the popover to the place we clicked, and show it.
                        decoded_view.context_menu.set_pointing_to(&get_rect_for_popover(tree_view, Some(button.get_position())));
                        decoded_view.context_menu.popup();
                    }
                    Inhibit(false)
                }
            ));

            // We we change the selection, we enable or disable the different actions of the Contextual Menu.
            decoded_view.tree_view.connect_cursor_changed(clone!(
                app_ui,
                copy_cell,
                copy_rows,
                paste_cell,
                delete_rows => move |tree_view| {

                    // If we have something selected, enable these actions.
                    if tree_view.get_selection().count_selected_rows() > 0 {
                        copy_cell.set_enabled(true);
                        copy_rows.set_enabled(true);
                        delete_rows.set_enabled(true);
                    }

                    // Otherwise, disable them.
                    else {
                        copy_cell.set_enabled(false);
                        copy_rows.set_enabled(false);
                        delete_rows.set_enabled(false);
                    }

                    // If we got text in the `Clipboard`...
                    if app_ui.clipboard.wait_for_text().is_some() {

                        // Get the selected cell, if any.
                        let selected_cell = tree_view.get_cursor();

                        // If we have a cell selected and it's not in the index column, enable "Paste Cell".
                        if selected_cell.0.is_some() {
                            if let Some(column) = selected_cell.1 {
                                if column.get_sort_column_id() > 0 {
                                    paste_cell.set_enabled(true);
                                }

                                // If the cell is invalid, disable the copy for it.
                                else if column.get_sort_column_id() < 0 {
                                    copy_cell.set_enabled(false);
                                    paste_cell.set_enabled(false);
                                }
                                else {
                                    paste_cell.set_enabled(false);
                                }
                            }
                            else { paste_cell.set_enabled(false); }
                        }
                        else { paste_cell.set_enabled(false); }
                    }
                    else { paste_cell.set_enabled(false); }
                }
            ));

            // When we hit the "Add row" button.
            add_rows.connect_activate(clone!(
                app_ui,
                decoded_view => move |_,_| {

                    // We hide the context menu.
                    decoded_view.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if decoded_view.tree_view.has_focus() {

                        // First, we check if the input is a valid number, as I'm already seeing people
                        // trying to add "two" rows.
                        match decoded_view.add_rows_entry.get_buffer().get_text().parse::<u32>() {

                            // If the number is valid...
                            Ok(number_rows) => {

                                // For each new row we want...
                                for _ in 0..number_rows {

                                    // Add a new empty line.
                                    decoded_view.list_store.insert_with_values(None, &[0, 1, 2, 3], &[&"New".to_value(), &"".to_value(), &"".to_value(), &true.to_value()]);
                                }
                            }
                            Err(error) => show_dialog(&app_ui.window, false, format!("You can only add an \"ENTIRE NUMBER\" of rows. Like 4, or 6. Maybe 5, who knows? But definetly not \"{}\".", Error::from(error).cause())),
                        }
                    }
                }
            ));

            // When we hit the "Delete row" button.
            delete_rows.connect_activate(clone!(
                app_ui,
                pack_file,
                packed_file_decoded,
                packed_file_decoded_index,
                decoded_view => move |_,_| {

                    // We hide the context menu.
                    decoded_view.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if decoded_view.tree_view.has_focus() {

                        // Get the selected row's `TreePath`.
                        let selected_rows = decoded_view.tree_view.get_selection().get_selected_rows().0;

                        // If we have any row selected...
                        if !selected_rows.is_empty() {

                            // For each row (in reverse)...
                            for row in (0..selected_rows.len()).rev() {

                                // Remove it.
                                decoded_view.list_store.remove(&decoded_view.list_store.get_iter(&selected_rows[row]).unwrap());
                            }

                            // Replace the old encoded data with the new one.
                            packed_file_decoded.borrow_mut().packed_file_data = PackedFileLocTreeView::return_data_from_tree_view(&decoded_view.list_store);

                            // Update the PackFile to reflect the changes.
                            update_packed_file_data_loc(
                                &*packed_file_decoded.borrow_mut(),
                                &mut *pack_file.borrow_mut(),
                                packed_file_decoded_index
                            );

                            // Set the mod as "Modified".
                            set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());
                        }
                    }
                }
            ));

            // When we hit the "Copy cell" button.
            copy_cell.connect_activate(clone!(
                app_ui,
                decoded_view => move |_,_| {

                    // Hide the context menu.
                    decoded_view.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if decoded_view.tree_view.has_focus() {

                        // Get the the focused cell.
                        let focused_cell = decoded_view.tree_view.get_cursor();

                        // If there is a focused `TreePath`...
                        if let Some(tree_path) = focused_cell.0 {

                            // And a focused `TreeViewColumn`...
                            if let Some(column) = focused_cell.1 {

                                // If the column is not a dummy one.
                                if column.get_sort_column_id() >= 0 {

                                    // If the cell is the index...
                                    if column.get_sort_column_id() == 0 {

                                        // Get his value and put it into the `Clipboard`.
                                        app_ui.clipboard.set_text(&decoded_view.list_store.get_value(&decoded_view.list_store.get_iter(&tree_path).unwrap(), 0).get::<String>().unwrap());
                                    }

                                    // If we are trying to copy the "tooltip" column...
                                    else if column.get_sort_column_id() == 3 {

                                        // Get the state of the toggle into an `&str`.
                                        let state = if decoded_view.list_store.get_value(&decoded_view.list_store.get_iter(&tree_path).unwrap(), 3).get().unwrap() { "true" } else { "false" };

                                        // Put the state of the toggle into the `Clipboard`.
                                        app_ui.clipboard.set_text(state);
                                    }

                                    // Otherwise...
                                    else {

                                        // Get the text from the focused cell and put it into the `Clipboard`.
                                        app_ui.clipboard.set_text(
                                            decoded_view.list_store.get_value(
                                                &decoded_view.list_store.get_iter(&tree_path).unwrap(),
                                                column.get_sort_column_id(),
                                            ).get::<&str>().unwrap()
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            ));

            // When we hit the "Paste cell" button.
            paste_cell.connect_activate(clone!(
                app_ui,
                pack_file,
                packed_file_decoded,
                packed_file_decoded_index,
                decoded_view => move |_,_| {

                    // Hide the context menu.
                    decoded_view.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if decoded_view.tree_view.has_focus() {

                        // Get the the focused cell.
                        let focused_cell = decoded_view.tree_view.get_cursor();

                        // If there is a focused `TreePath`...
                        if let Some(tree_path) = focused_cell.0 {

                            // And a focused `TreeViewColumn`...
                            if let Some(column) = focused_cell.1 {

                                // If the cell is the index...
                                if column.get_sort_column_id() > 0 {

                                    // If we are trying to paste the "tooltip" column...
                                    if column.get_sort_column_id() == 3 {

                                        // If we got the state of the toggle from the `Clipboard`...
                                        if let Some(data) = app_ui.clipboard.wait_for_text() {

                                            // Get the state of the toggle into an `&str`.
                                            let state = if data == "true" { true } else if data == "false" { false } else { return show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a Loc PackedFile:\n\nThe value provided is neither \"true\" nor \"false\".") };

                                            // Set the state of the toggle of the cell.
                                            decoded_view.list_store.set_value(&decoded_view.list_store.get_iter(&tree_path).unwrap(), 3, &state.to_value());

                                            // Replace the old encoded data with the new one.
                                            packed_file_decoded.borrow_mut().packed_file_data = PackedFileLocTreeView::return_data_from_tree_view(&decoded_view.list_store);

                                            // Update the PackFile to reflect the changes.
                                            update_packed_file_data_loc(
                                                &*packed_file_decoded.borrow_mut(),
                                                &mut *pack_file.borrow_mut(),
                                                packed_file_decoded_index
                                            );

                                            // Set the mod as "Modified".
                                            set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());
                                        }
                                    }

                                    // Otherwise, if we got the state of the toggle from the `Clipboard`...
                                    else if let Some(data) = app_ui.clipboard.wait_for_text() {

                                        // Update his value.
                                        decoded_view.list_store.set_value(&decoded_view.list_store.get_iter(&tree_path).unwrap(), column.get_sort_column_id() as u32, &data.to_value());

                                        // Replace the old encoded data with the new one.
                                        packed_file_decoded.borrow_mut().packed_file_data = PackedFileLocTreeView::return_data_from_tree_view(&decoded_view.list_store);

                                        // Update the PackFile to reflect the changes.
                                        update_packed_file_data_loc(
                                            &*packed_file_decoded.borrow_mut(),
                                            &mut *pack_file.borrow_mut(),
                                            packed_file_decoded_index
                                        );

                                        // Set the mod as "Modified".
                                        set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());
                                    }
                                }
                            }
                        }
                    }
                }
            ));

            // When we hit the "Copy row" button.
            copy_rows.connect_activate(clone!(
                app_ui,
                decoded_view => move |_,_| {

                    // Hide the context menu.
                    decoded_view.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if decoded_view.tree_view.has_focus() {

                        // Get the selected rows.
                        let selected_rows = decoded_view.tree_view.get_selection().get_selected_rows().0;

                        // If there is something selected...
                        if !selected_rows.is_empty() {

                            // Get the list of `TreeIter`s we want to copy.
                            let tree_iter_list = selected_rows.iter().map(|row| decoded_view.list_store.get_iter(row).unwrap()).collect::<Vec<TreeIter>>();

                            // Create the `String` that will copy the row that will bring that shit of TLJ down.
                            let mut copy_string = String::new();

                            // For each row...
                            for row in &tree_iter_list {

                                // Get the data from the three columns, and push it to our copy `String`.
                                copy_string.push_str(decoded_view.list_store.get_value(row, 1).get::<&str>().unwrap());
                                copy_string.push('\t');
                                copy_string.push_str(decoded_view.list_store.get_value(row, 2).get::<&str>().unwrap());
                                copy_string.push('\t');
                                copy_string.push_str(
                                    match decoded_view.list_store.get_value(row, 3).get::<bool>().unwrap() {
                                        true => "true",
                                        false => "false",
                                    }
                                );
                                copy_string.push('\n');
                            }

                            // Pass all the copied rows to the clipboard.
                            app_ui.clipboard.set_text(&copy_string);
                        }
                    }
                }
            ));

            // When we hit the "Paste row" button.
            paste_rows.connect_activate(clone!(
                app_ui,
                pack_file,
                packed_file_decoded,
                packed_file_decoded_index,
                decoded_view => move |_,_| {

                    // Hide the context menu.
                    decoded_view.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if decoded_view.tree_view.has_focus() {

                        // Before anything else, we check if the data in the `Clipboard` includes ONLY valid rows.
                        if check_clipboard_row(&app_ui) {

                            // When it gets the data from the `Clipboard`....
                            if let Some(data) = app_ui.clipboard.wait_for_text() {

                                // Store here all the decoded fields.
                                let mut fields_data = vec![];

                                // For each row in the data we received...
                                for row in data.lines() {

                                    // Get all the data from his fields.
                                    fields_data.push(row.split('\t').map(|x| x.to_owned()).collect::<Vec<String>>());
                                }

                                // Get the selected row, if there is any.
                                let selected_row = decoded_view.tree_view.get_selection().get_selected_rows().0;

                                // If there is at least one line selected, use it as "base" to paste.
                                let mut tree_iter = if !selected_row.is_empty() {
                                    decoded_view.list_store.get_iter(&selected_row[0]).unwrap()
                                }

                                // Otherwise, append a new `TreeIter` to the `TreeView`, and use it.
                                else { decoded_view.list_store.append() };

                                // For each row in our fields_data vec...
                                for (row_index, row) in fields_data.iter().enumerate() {

                                    // Fill the "Index" column with "New".
                                    decoded_view.list_store.set_value(&tree_iter, 0, &"New".to_value());

                                    // Fill the "key" and "text" columns.
                                    decoded_view.list_store.set_value(&tree_iter, 1, &row[0].to_value());
                                    decoded_view.list_store.set_value(&tree_iter, 2, &row[1].to_value());

                                    // Fill the "tooltip" column.
                                    decoded_view.list_store.set_value(&tree_iter, 3, &(if row[2] == "true" { true } else {false}).to_value());

                                    // Move to the next row. If it doesn't exist and it's not the last loop....
                                    if !decoded_view.list_store.iter_next(&tree_iter) && row_index < (fields_data.len() - 1) {

                                        // Create it.
                                        tree_iter = decoded_view.list_store.append();
                                    }
                                }

                                // Replace the old encoded data with the new one.
                                packed_file_decoded.borrow_mut().packed_file_data = PackedFileLocTreeView::return_data_from_tree_view(&decoded_view.list_store);

                                // Update the PackFile to reflect the changes.
                                update_packed_file_data_loc(
                                    &*packed_file_decoded.borrow_mut(),
                                    &mut *pack_file.borrow_mut(),
                                    packed_file_decoded_index
                                );

                                // Set the mod as "Modified".
                                set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());
                            };
                        }
                    }
                }
            ));

            // When we hit the "Import to TSV" button.
            import_tsv.connect_activate(clone!(
                app_ui,
                pack_file,
                packed_file_decoded,
                packed_file_decoded_index,
                decoded_view => move |_,_|{

                    // We hide the context menu.
                    decoded_view.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if decoded_view.tree_view.has_focus() {

                        // Create the `FileChooser`.
                        let file_chooser = FileChooserNative::new(
                            "Select TSV File to Import...",
                            &app_ui.window,
                            FileChooserAction::Open,
                            "Import",
                            "Cancel"
                        );

                        // Enable the TSV filter for the `FileChooser`.
                        file_chooser_filter_packfile(&file_chooser, "*.tsv");

                        // If we have selected a file to import...
                        if file_chooser.run() == gtk_response_accept {

                            // If there is an error while importing the TSV file, we report it.
                            if let Err(error) = packed_file_decoded.borrow_mut().packed_file_data.import_tsv(
                                &file_chooser.get_filename().unwrap(),
                                "Loc PackedFile"
                            ) { return show_dialog(&app_ui.window, false, error.cause()); }

                            // Load the new data to the TreeView.
                            PackedFileLocTreeView::load_data_to_tree_view(&packed_file_decoded.borrow().packed_file_data, &decoded_view.list_store);

                            // Update the PackFile to reflect the changes.
                            update_packed_file_data_loc(
                                &*packed_file_decoded.borrow_mut(),
                                &mut *pack_file.borrow_mut(),
                                packed_file_decoded_index
                            );

                            // Set the mod as "Modified".
                            set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());
                        }
                    }
                }
            ));

            // When we hit the "Export to TSV" button.
            export_tsv.connect_activate(clone!(
                app_ui,
                packed_file_decoded,
                decoded_view => move |_,_|{

                    // We hide the context menu.
                    decoded_view.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if decoded_view.tree_view.has_focus() {

                        // Create the `FileChooser`.
                        let file_chooser = FileChooserNative::new(
                            "Export TSV File...",
                            &app_ui.window,
                            FileChooserAction::Save,
                            "Save",
                            "Cancel"
                        );

                        // We want to ask before overwriting files. Just in case. Otherwise, there can be an accident.
                        file_chooser.set_do_overwrite_confirmation(true);

                        // Set the name of the Loc PackedFile as the default new name.
                        file_chooser.set_current_name(format!("{}.tsv", &tree_path.last().unwrap()));

                        // If we hit "Save"...
                        if file_chooser.run() == gtk_response_accept {

                            // Try to export the TSV.
                            match packed_file_decoded.borrow_mut().packed_file_data.export_tsv(
                                &file_chooser.get_filename().unwrap(),
                                ("Loc PackedFile", 9001)
                            ) {
                                Ok(result) => show_dialog(&app_ui.window, true, result),
                                Err(error) => show_dialog(&app_ui.window, false, error.cause())
                            }
                        }
                    }
                }
            ));
        }

        // Things that happen when we edit a cell.
        {

            // When we edit the "Key" column.
            decoded_view.cell_key.connect_edited(clone!(
                app_ui,
                pack_file,
                packed_file_decoded,
                packed_file_decoded_index,
                decoded_view => move |_, tree_path, new_text|{

                    // Get the cell's old text, to check for changes.
                    let tree_iter = decoded_view.list_store.get_iter(&tree_path).unwrap();
                    let old_text: String = decoded_view.list_store.get_value(&tree_iter, 1).get().unwrap();

                    // If the text has changed we need to check that the new text is valid, as this is a key column.
                    // Otherwise, we do nothing.
                    if old_text != new_text && !new_text.is_empty() && !new_text.contains(' ') {

                        // Get the first row's `TreeIter`.
                        let current_line = decoded_view.list_store.get_iter_first().unwrap();

                        // Loop to search for coincidences.
                        let mut key_already_exists = false;
                        loop {

                            //  If we found a coincidence, break the loop.
                            if decoded_view.list_store.get_value(&current_line, 1).get::<String>().unwrap() == new_text {
                                key_already_exists = true;
                                break;
                            }

                            // If we reached the end of the `ListStore`, we break the loop.
                            else if !decoded_view.list_store.iter_next(&current_line) { break; }
                        }

                        // If there is a coincidence with another key...
                        if key_already_exists {
                            show_dialog(&app_ui.window, false, "This key is already in the Loc PackedFile.");
                        }

                        // If it has passed all the checks without error...
                        else {

                            // Change the value in the cell.
                            decoded_view.list_store.set_value(&tree_iter, 1, &new_text.to_value());

                            // Replace the old encoded data with the new one.
                            packed_file_decoded.borrow_mut().packed_file_data = PackedFileLocTreeView::return_data_from_tree_view(&decoded_view.list_store);

                            // Update the PackFile to reflect the changes.
                            update_packed_file_data_loc(
                                &*packed_file_decoded.borrow_mut(),
                                &mut *pack_file.borrow_mut(),
                                packed_file_decoded_index
                            );

                            // Set the mod as "Modified".
                            set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());
                        }
                    }

                    // If the field is empty,
                    else if new_text.is_empty() {
                        show_dialog(&app_ui.window, false, "Only my hearth can be empty.");
                    }

                    // If the field contains spaces.
                    else if new_text.contains(' ') {
                        show_dialog(&app_ui.window, false, "Spaces are not valid characters.");
                    }
                }
            ));

            // When we edit the "Text" column.
            decoded_view.cell_text.connect_edited(clone!(
                app_ui,
                pack_file,
                packed_file_decoded,
                packed_file_decoded_index,
                decoded_view => move |_, tree_path, new_text| {

                    // Get the cell's old text, to check for changes.
                    let tree_iter = decoded_view.list_store.get_iter(&tree_path).unwrap();
                    let old_text: String = decoded_view.list_store.get_value(&tree_iter, 2).get().unwrap();

                    // If it has changed...
                    if old_text != new_text {

                        // Change the value in the cell.
                        decoded_view.list_store.set_value(&tree_iter, 2, &new_text.to_value());

                        // Replace the old encoded data with the new one.
                        packed_file_decoded.borrow_mut().packed_file_data = PackedFileLocTreeView::return_data_from_tree_view(&decoded_view.list_store);

                        // Update the PackFile to reflect the changes.
                        update_packed_file_data_loc(
                            &*packed_file_decoded.borrow_mut(),
                            &mut *pack_file.borrow_mut(),
                            packed_file_decoded_index
                        );

                        // Set the mod as "Modified".
                        set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());
                    }
                }
            ));

            // When we change the state (true/false) of the "Tooltip" cell.
            decoded_view.cell_tooltip.connect_toggled(clone!(
                app_ui,
                pack_file,
                packed_file_decoded,
                packed_file_decoded_index,
                decoded_view => move |cell, tree_path|{

                    // Get his `TreeIter` and his column.
                    let tree_iter = decoded_view.list_store.get_iter(&tree_path).unwrap();
                    let edited_cell_column = decoded_view.tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                    // Get his new state.
                    let state = !cell.get_active();

                    // Change it in the `ListStore`.
                    decoded_view.list_store.set_value(&tree_iter, edited_cell_column, &state.to_value());

                    // Change his state.
                    cell.set_active(state);

                    // Replace the old encoded data with the new one.
                    packed_file_decoded.borrow_mut().packed_file_data = PackedFileLocTreeView::return_data_from_tree_view(&decoded_view.list_store);

                    // Update the PackFile to reflect the changes.
                    update_packed_file_data_loc(
                        &*packed_file_decoded.borrow_mut(),
                        &mut *pack_file.borrow_mut(),
                        packed_file_decoded_index
                    );

                    // Set the mod as "Modified".
                    set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());
                }
            ));
        }
    }

    /// This function loads the data from a `LocData` into a `TreeView`.
    pub fn load_data_to_tree_view(
        packed_file_data: &LocData,
        packed_file_list_store: &ListStore
    ) {
        // First, we delete all the data from the `ListStore`. Just in case there is something there.
        packed_file_list_store.clear();

        // Then we add every line to the ListStore.
        for (j, i) in packed_file_data.packed_file_data_entries.iter().enumerate() {
            packed_file_list_store.insert_with_values(None, &[0, 1, 2, 3], &[&format!("{:0count$}", j + 1, count = (packed_file_data.packed_file_data_entries.len().to_string().len() + 1)), &i.key, &i.text, &i.tooltip]);
        }
    }

    /// This function returns a `LocData` with all the stuff in the table. We need for it the `ListStore` of that table.
    pub fn return_data_from_tree_view(
        list_store: &ListStore,
    ) -> LocData {

        // Create an empty `LocData`.
        let mut loc_data = LocData::new();

        // If we got at least one row...
        if let Some(current_line) = list_store.get_iter_first() {

            // Loop 'til the end of the storm of the sword and axe.
            loop {

                // Make a new entry with the data from the `ListStore`, and push it to our new `LocData`.
                loc_data.packed_file_data_entries.push(
                    LocDataEntry::new(
                        list_store.get_value(&current_line, 1).get().unwrap(),
                        list_store.get_value(&current_line, 2).get().unwrap(),
                        list_store.get_value(&current_line, 3).get().unwrap(),
                    )
                );

                // If there are no more rows, stop, for the wolf has come.
                if !list_store.iter_next(&current_line) { break; }
            }
        }

        // Return the new `LocData`.
        loc_data
    }
}

/// This function checks if the data in the clipboard is a valid row of a Loc PackedFile. Returns
/// `true` if the data in the clipboard forms valid rows, and `false` if any of them is an invalid row.
fn check_clipboard_row(app_ui: &AppUI) -> bool {

    // Try to get the data from the `Clipboard`....
    if let Some(data) = app_ui.clipboard.wait_for_text() {

        // Store here all the decoded fields.
        let mut fields_data = vec![];

        // For each row in the data we received...
        for row in data.lines() {

            // Get all the data from his fields.
            fields_data.push(row.split('\t').map(|x| x.to_owned()).collect::<Vec<String>>());
        }

        // If we at least have one row...
        if !fields_data.is_empty() {

            // Var to control when a field is invalid.
            let mut data_is_invalid = false;

            // For each row we have...
            for row in &fields_data {

                // If we have the same amount of data for each field...
                if row.len() == 3 {

                    // If the third field is not valid, the data is invalid.
                    if row[2] != "true" && row[2] != "false" {
                        data_is_invalid = true;
                        break;
                    }
                }

                // Otherwise, the rows are invalid.
                else {
                    data_is_invalid = true;
                    break;
                }
            }

            // If in any point the data was invalid, return false.
            if data_is_invalid { false } else { true }
        }

        // Otherwise, the contents of the `Clipboard` are invalid.
        else { false }
    }

    // Otherwise, there is no data in the `Clipboard`.
    else { false }
}
