// In this file are all the helper functions used by the UI when decoding Loc PackedFiles.
extern crate gtk;

use gtk::prelude::*;
use gtk::{
    TreeView, ListStore, ScrolledWindow, Popover, Entry, ModelButton,
    CellRendererText, TreeViewColumn, CellRendererToggle, Separator, Orientation, Grid,
    TreeViewColumnSizing, TreeViewGridLines, EntryIconPosition
};

use packedfile::loc::LocData;
use packedfile::loc::LocDataEntry;
use settings::*;
use ui::*;

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
    pub fn create_tree_view(packed_file_data_display: &Grid, settings: &Settings) -> PackedFileLocTreeView {

        // First, we create the new `TreeView` and his `ListStore`.
        let tree_view = TreeView::new();
        let list_store = ListStore::new(&[String::static_type(), String::static_type(), String::static_type(), gtk::Type::Bool]);

        // Config the `TreeView`.
        tree_view.set_model(Some(&list_store));
        tree_view.set_grid_lines(TreeViewGridLines::Both);
        tree_view.set_rubber_banding(true);
        tree_view.set_enable_search(false);
        tree_view.set_search_column(1);
        tree_view.set_margin_bottom(10);

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
        packed_file_data_display.attach(&packed_file_data_scroll, 0, 0, 1, 1);
        packed_file_data_display.show_all();

        // Hide the Context Menu by default.
        context_menu.hide();

        // Actions that we could move here from the `main.rs`.

        // When we right-click the `TreeView`, show the Contextual Menu.
        //
        // NOTE: REMEMBER, WE OPEN THE POPUP HERE, BUT WE NEED TO CLOSED IT WHEN WE HIT HIS BUTTONS.
        tree_view.connect_button_release_event(clone!(
            context_menu => move |tree_view, button| {

                // If we clicked the right mouse button...
                if button.get_button() == 3 {

                    // Point the popover to the place we clicked, and show it.
                    context_menu.set_pointing_to(&get_rect_for_popover(tree_view, Some(button.get_position())));
                    context_menu.popup();
                }
                Inhibit(false)
            }
        ));

        // Return the struct with the Loc `TreeView` and all the stuff we want.
        PackedFileLocTreeView {
            tree_view,
            list_store,
            cell_key,
            cell_text,
            cell_tooltip,
            context_menu,
            add_rows_entry,
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
