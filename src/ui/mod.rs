// In this file are all the helper functions used by the UI (mainly GTK here)
extern crate num;
extern crate url;
extern crate qt_widgets;
extern crate qt_gui;
extern crate qt_core;
extern crate cpp_utils;

use qt_widgets::application::Application;
use qt_widgets::widget::Widget;
use qt_widgets::grid_layout::GridLayout;
use qt_widgets::splitter::Splitter;
use qt_widgets::tree_view::TreeView;
use qt_widgets::main_window::MainWindow;
use qt_widgets::message_box::MessageBox;
use qt_widgets::message_box::Icon;
use qt_widgets::message_box::StandardButton;
use qt_widgets::action_group::ActionGroup;
use qt_widgets::label::Label;
use qt_core::item_selection::ItemSelection;
use qt_core::flags::Flags;

use qt_gui::standard_item_model::StandardItemModel;
use qt_gui::standard_item::StandardItem;
use qt_core::item_selection_model::ItemSelectionModel;
use qt_core::event_loop::EventLoop;
use qt_core::connection::Signal;
use qt_core::variant::Variant;
use qt_core::slots::SlotBool;
use qt_core::object::Object;
use cpp_utils::{CppBox, StaticCast, DynamicCast};

use url::Url;
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::fmt::Display;

use QString;








pub mod settings;
pub mod updater;

use AppUI;
use common::*;
/*




use packedfile::*;
use packedfile::db::*;
use packedfile::loc::*;
use packedfile::db::schemas::Schema;
use packfile::packfile::PackFile;
use packfile::packfile::PackedFile;

pub mod packedfile_db;
pub mod packedfile_loc;
pub mod packedfile_text;
pub mod packedfile_image;
pub mod packedfile_rigidmodel;

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
    about_dialog.add_credit_section("LUA functions by", &["Aexrael Dex"]);
    about_dialog.add_credit_section("Windows's theme", &["\"Materia for GTK3\" by nana-4"]);
    about_dialog.add_credit_section("Text Editor theme", &["\"Monokai Extended\" by Leo Iannacone"]);
    about_dialog.add_credit_section("Special thanks to", &["- PFM team (for providing the community\n   with awesome modding tools).", "- CA (for being a mod-friendly company)."]);

    // Center the `AboutDialog` in the middle of the screen.
    about_dialog.set_position(WindowPosition::CenterOnParent);

    // Give a father to the poor orphan...
    about_dialog.set_transient_for(parent_window);

    // Run the `AboutDialog`.
    about_dialog.run();
    about_dialog.destroy();
}

/// This function creates an `ApplicationWindow`, asking for a name for a PakcedFile. Also, sets the
/// events to control his buttons.
pub fn show_create_packed_file_window(
    application: &Application,
    app_ui: &AppUI,
    rpfm_path: &PathBuf,
    pack_file: &Rc<RefCell<PackFile>>,
    packed_file_type: PackedFileType,
    dependency_database: &Rc<RefCell<Option<Vec<PackedFile>>>>,
    schema: &Rc<RefCell<Option<Schema>>>,
) {

    // Create the new ApplicationWindow.
    let window = ApplicationWindow::new(application);
    window.set_transient_for(&app_ui.window);
    window.set_position(WindowPosition::CenterOnParent);
    window.set_icon_from_file(&Path::new(&format!("{}/img/rpfm.png", rpfm_path.to_string_lossy()))).unwrap();

    // Depending on the type of PackedFile we want to create, set the title.
    match packed_file_type {
        PackedFileType::Loc => window.set_title("Create Loc File"),
        PackedFileType::DB => window.set_title("Create DB Table"),
        PackedFileType::Text => window.set_title("Create Text File"),
    }

    // Disable the menubar in this window.
    window.set_show_menubar(false);

    // Create the grid to pack all the stuff.
    let grid = Grid::new();
    grid.set_border_width(6);
    grid.set_row_spacing(3);
    grid.set_column_spacing(3);

    // Create the text entry for the name of the file.
    let entry = Entry::new();
    entry.set_size_request(300, 0);
    entry.set_has_frame(false);
    entry.set_hexpand(true);
    if let PackedFileType::DB = packed_file_type {
        entry.set_placeholder_text("Write the name of your table here.");
    }
    else { entry.set_placeholder_text("Write the full path of the PackFile here."); }

    // Get the selected path and put it in the entry.
    let path = get_tree_path_from_selection(&app_ui.folder_tree_selection, false);
    let mut path_string = path.iter().map(|x| format!("{}/", x)).collect::<String>();

    // Depending on the type of PackedFile we want to create, set the default name. In case of tables,
    // we don't allow to put a path, as it depends on the selected table.
    match packed_file_type {
        PackedFileType::Loc => path_string.push_str("new_file.loc"),
        PackedFileType::DB => path_string = "new_file".to_owned(),
        PackedFileType::Text => path_string.push_str("new_file.lua"),
    }

    entry.set_text(&path_string);

    // Create and populate the Table Selector.
    let table_label = Label::new(Some("Default Game Selected:"));
    let table_combo = ComboBoxText::new();
    table_label.set_size_request(120, 0);
    table_label.set_xalign(0.0);
    table_label.set_yalign(0.5);

    // If it's a table, we try to populate the combo.
    if let PackedFileType::DB = packed_file_type {

        // Only if there is a dependency_database we populate this.
        if let Some(ref dependency_database) = *dependency_database.borrow() {
            for table in dependency_database.iter() {
                table_combo.append(Some(&*table.path[1]), &*table.path[1]);
            }
        }
    }

    // Otherwise, add none, so it doesn't crash when calling it for another PackedFile Type.
    else { table_combo.append(Some("none"), "none"); }

    table_combo.set_active(0);
    table_combo.set_hexpand(true);

    // Create the bottom ButtonBox
    let button_box = ButtonBox::new(Orientation::Horizontal);
    let cancel_button = Button::new_with_label("Cancel");
    let accept_button = Button::new_with_label("Accept");

    button_box.pack_start(&cancel_button, false, false, 0);
    button_box.pack_start(&accept_button, false, false, 0);
    button_box.set_layout(ButtonBoxStyle::Spread);
    button_box.set_spacing(10);

    // Pack all the stuff in the grid.
    grid.attach(&entry, 0, 0, 2, 1);
    grid.attach(&button_box, 0, 2, 2, 1);

    // If we are creating a DB Table, we pack the table selector too.
    if let PackedFileType::DB = packed_file_type {
        grid.attach(&table_label, 0, 1, 1, 1);
        grid.attach(&table_combo, 1, 1, 1, 1);
    }

    // Add the grid to the window and show it.
    window.add(&grid);
    window.show_all();

    // Get the written path.
    let path = match packed_file_type {
        PackedFileType::DB => vec!["db".to_owned(), table_combo.get_active_id().unwrap(), entry.get_text().unwrap()],
        _ => entry.get_text().unwrap().split('/').map(|x| x.to_owned()).collect::<Vec<String>>(),
    };

    // We check if the file already exists. If it exists, disable the "Accept" button.
    if pack_file.borrow().data.packedfile_exists(&path) { accept_button.set_sensitive(false); }

    // Otherwise, enable it.
    else { accept_button.set_sensitive(true); }

    // Disable the main window so you can't use it with this window open.
    app_ui.window.set_sensitive(false);

    // When we change the selected table.
    table_combo.connect_changed(clone!(
        accept_button,
        pack_file,
        entry => move |table_combo| {

            // Get the written path.
            let path = vec!["db".to_owned(), table_combo.get_active_id().unwrap(), entry.get_text().unwrap()];

            // We check if the file already exists. If it exists, disable the "Accept" button.
            if pack_file.borrow().data.packedfile_exists(&path) { accept_button.set_sensitive(false); }

            // Otherwise, enable it.
            else { accept_button.set_sensitive(true); }
        }
    ));

    // When we change the name in the entry.
    entry.connect_changed(clone!(
        packed_file_type,
        accept_button,
        table_combo,
        pack_file => move |entry| {

            // Depending on what type of file we have, we need to make one check or another.
            match packed_file_type {

                // If it's a Loc PackedFile...
                PackedFileType::Loc => {

                    // Get the written path.
                    let path = entry.get_text().unwrap().split('/').map(|x| x.to_owned()).collect::<Vec<String>>();

                    // If the path contains empty fields, is invalid, no matter what else it has.
                    if path.contains(&String::new()) { accept_button.set_sensitive(false); }

                    // Otherwise...
                    else {

                        // If the name it's empty (path ends in '/', or len 1 and 0 is empty), disable the button.
                        if path.last().unwrap().is_empty() { accept_button.set_sensitive(false); }

                        // Otherwise...
                        else {

                            // Get his name.
                            if let Some(name) = path.last() {

                                // If ends in ".loc" the name is valid, so we check if exists.
                                if name.ends_with(".loc") {

                                    // If the path exists, disable the button.
                                    if pack_file.borrow().data.packedfile_exists(&path) { accept_button.set_sensitive(false); }

                                    // Otherwise, enable it.
                                    else { accept_button.set_sensitive(true); }
                                }

                                // If the name doesn't end in ".loc", we fix it and check if the fixed name exists.
                                else {

                                    // Get a mutable copy of the path.
                                    let mut fixed_path = path.to_vec();

                                    // Get his current name.
                                    let name = fixed_path.pop().unwrap();

                                    // Change it.
                                    fixed_path.push(format!("{}.loc", name));

                                    // If the path exists, disable the button.
                                    if pack_file.borrow().data.packedfile_exists(&fixed_path) { accept_button.set_sensitive(false); }

                                    // Otherwise, enable it.
                                    else { accept_button.set_sensitive(true); }
                                }
                            }

                            // If there is an error, disable the button.
                            else { accept_button.set_sensitive(false); }
                        }
                    }
                },

                // If it's a DB PackedFile...
                PackedFileType::DB => {

                    // Replace his path with one inside the table's directory.
                    let path = vec!["db".to_owned(), table_combo.get_active_id().unwrap(), entry.get_text().unwrap()];

                    // If the name it's empty, disable the button.
                    if path.last().unwrap().is_empty() { accept_button.set_sensitive(false); }

                    // If the path exists, disable the button.
                    else if pack_file.borrow().data.packedfile_exists(&path) { accept_button.set_sensitive(false); }

                    // Otherwise, enable it.
                    else { accept_button.set_sensitive(true); }
                },

                // If it's a Text PackedFile...
                PackedFileType::Text => {

                    // Get the written path.
                    let path = entry.get_text().unwrap().split('/').map(|x| x.to_owned()).collect::<Vec<String>>();

                    // If the path contains empty fields, is invalid, no matter what else it has.
                    if path.contains(&String::new()) { accept_button.set_sensitive(false); }

                    // Otherwise...
                    else {

                        // If the name it's empty (path ends in '/', or len 1 and 0 is empty), disable the button.
                        if path.last().unwrap().is_empty() { accept_button.set_sensitive(false); }

                        // Otherwise...
                        else {

                            // Get his name.
                            if let Some(name) = path.last() {

                                // If ends in something valid, the name is valid, so we check if exists.
                                if name.ends_with(".lua") ||
                                    name.ends_with(".xml") ||
                                    name.ends_with(".xml.shader") ||
                                    name.ends_with(".xml.material") ||
                                    name.ends_with(".variantmeshdefinition") ||
                                    name.ends_with(".environment") ||
                                    name.ends_with(".lighting") ||
                                    name.ends_with(".wsmodel") ||
                                    name.ends_with(".csv") ||
                                    name.ends_with(".tsv") ||
                                    name.ends_with(".inl") ||
                                    name.ends_with(".battle_speech_camera") ||
                                    name.ends_with(".bob") ||
                                    name.ends_with(".txt") {

                                    // If the path exists, disable the button.
                                    if pack_file.borrow().data.packedfile_exists(&path) { accept_button.set_sensitive(false); }

                                    // Otherwise, enable it.
                                    else { accept_button.set_sensitive(true); }
                                }

                                // If the name doesn't end in something valid, we fix it and check if the fixed name exists.
                                else {

                                    // Get a mutable copy of the path.
                                    let mut fixed_path = path.to_vec();

                                    // Get his current name.
                                    let name = fixed_path.pop().unwrap();

                                    // Change it.
                                    fixed_path.push(format!("{}.lua", name));

                                    // If the path exists, disable the button.
                                    if pack_file.borrow().data.packedfile_exists(&fixed_path) { accept_button.set_sensitive(false); }

                                    // Otherwise, enable it.
                                    else { accept_button.set_sensitive(true); }
                                }
                            }

                            // If there is an error, disable the button.
                            else { accept_button.set_sensitive(false); }
                        }
                    }
                },
            }
        }
    ));

    // When we press the "Accept" button.
    accept_button.connect_button_release_event(clone!(
        dependency_database,
        packed_file_type,
        table_combo,
        pack_file,
        schema,
        window,
        entry,
        app_ui => move |_,_| {

            // Try to create the PackedFile.
            let path = match create_packed_file(
                &entry.get_text().unwrap(),
                &table_combo.get_active_id().unwrap(),
                &schema.borrow(),
                &mut pack_file.borrow_mut(),
                &dependency_database.borrow(),
                &packed_file_type,
            ) {
                Ok(path) => path,
                Err(error) => {
                    show_dialog(&app_ui.window, false, error.cause());
                    return Inhibit(false)
                }
            };

            // Set the mod as "Modified".
            set_modified(true, &app_ui.window, &mut pack_file.borrow_mut());

            // Update the TreeView to show the newly added PackedFile.
            update_treeview(
                &app_ui.folder_tree_store,
                &pack_file.borrow(),
                &app_ui.folder_tree_selection,
                TreeViewOperation::Add(path.to_vec()),
                &TreePathType::None,
            );

            // Destroy the "Settings Window".
            window.destroy();

            // Re-enable the main window.
            app_ui.window.set_sensitive(true);

            Inhibit(false)
        }
    ));

    // When we press the "Cancel" button, we close the window.
    cancel_button.connect_button_release_event(clone!(
        window,
        app_ui => move |_,_| {

            // Destroy the "Settings Window".
            window.destroy();

            // Re-enable the main window.
            app_ui.window.set_sensitive(true);

            Inhibit(false)
        }
    ));

    // When we close the window.
    window.connect_delete_event(clone!(
        app_ui => move |window,_| {

            // Destroy the "Settings Window".
            window.destroy();

            // Re-enable the main window.
            app_ui.window.set_sensitive(true);

            Inhibit(false)
        }
    ));
}

/// This function creates an `ApplicationWindow`, asking for a name for the mass-imported TSV files.
/// Also, will create the events to control his buttons.
#[allow(dead_code)]
pub fn show_tsv_mass_import_window(
    application: &Application,
    app_ui: &AppUI,
    rpfm_path: &PathBuf,
    pack_file: &Rc<RefCell<PackFile>>,
    schema: &Rc<RefCell<Option<Schema>>>,
) {

    // Create the new ApplicationWindow.
    let window = ApplicationWindow::new(application);
    window.set_transient_for(&app_ui.window);
    window.set_position(WindowPosition::CenterOnParent);
    window.set_icon_from_file(&Path::new(&format!("{}/img/rpfm.png", rpfm_path.to_string_lossy()))).unwrap();
    window.set_title("Mass-Import TSV Files");

    // Disable the menubar in this window.
    window.set_show_menubar(false);

    // Create the grid to pack all the stuff.
    let grid = Grid::new();
    grid.set_border_width(6);
    grid.set_row_spacing(3);
    grid.set_column_spacing(3);

    // Create the text entry for the name of the file.
    let entry = Entry::new();
    entry.set_size_request(300, 0);
    entry.set_has_frame(false);
    entry.set_hexpand(true);
    entry.set_placeholder_text("Write the name used for your new tables here.");
    entry.set_text("new_table");

    // Create the "..." button for selecting TSV Files.
    let tsv_selector_button = Button::new_with_label("...");

    // Create the bottom ButtonBox
    let button_box = ButtonBox::new(Orientation::Horizontal);
    let cancel_button = Button::new_with_label("Cancel");
    let accept_button = Button::new_with_label("Accept");

    button_box.pack_start(&cancel_button, false, false, 0);
    button_box.pack_start(&accept_button, false, false, 0);
    button_box.set_layout(ButtonBoxStyle::Spread);
    button_box.set_spacing(10);

    // Pack all the stuff in the grid.
    grid.attach(&entry, 0, 0, 1, 1);
    grid.attach(&tsv_selector_button, 1, 0, 1, 1);
    grid.attach(&button_box, 0, 1, 2, 1);

    // Add the grid to the window and show it.
    window.add(&grid);
    window.show_all();

    // Disable the main window so you can't use it with this window open.
    app_ui.window.set_sensitive(false);

    // Disable the "Accept" button by default too.
    accept_button.set_sensitive(false);

    // Create the vector that'll hold the paths of the TSV files.
    let tsv_paths = Rc::new(RefCell::new(vec![]));

    // When we change the name in the entry.
    entry.connect_changed(clone!(
        accept_button => move |_| {

            // If it's stupid but it works,...
            accept_button.set_relief(ReliefStyle::None);
            accept_button.set_relief(ReliefStyle::Normal);
        }
    ));

    // When we press the "..." button.
    tsv_selector_button.connect_button_release_event(clone!(
        accept_button,
        tsv_paths,
        window => move |_,_| {

            // Create a FileChooser to select the TSV files.
            let file_chooser_select_tsv_files = FileChooserNative::new(
                "Select TSV Files...",
                &window,
                FileChooserAction::Open,
                "Accept",
                "Cancel"
            );

            // Allow to select multiple files at the same time.
            file_chooser_select_tsv_files.set_select_multiple(true);

            // Then run the created FileChooser, get all the selected URIS, turn them into paths and add them to the list.
            if file_chooser_select_tsv_files.run() == Into::<i32>::into(ResponseType::Accept) {
                for path in &file_chooser_select_tsv_files.get_uris() {
                    tsv_paths.borrow_mut().push(Url::parse(&path).unwrap().to_file_path().unwrap());
                }

                // If it's stupid but it works,...
                accept_button.set_relief(ReliefStyle::None);
                accept_button.set_relief(ReliefStyle::Normal);
            }

            Inhibit(false)
        }
    ));

    accept_button.connect_property_relief_notify(clone!(
        tsv_paths,
        entry => move |accept_button| {

            // If there is nothing in the entry, or the TSV list is empty, disable the "Accept" button.
            if entry.get_text().unwrap().is_empty() || tsv_paths.borrow().is_empty() {
                accept_button.set_sensitive(false);
            }

            // Otherwise, enable it.
            else { accept_button.set_sensitive(true); }

        }
    ));

    // When we press the "Accept" button.
    accept_button.connect_button_release_event(clone!(
        pack_file,
        tsv_paths,
        schema,
        window,
        entry,
        app_ui => move |_,_| {

            // Try to mass-import all the provided TSV files.
            let tree_paths = match tsv_mass_import(&tsv_paths.borrow(), &entry.get_text().unwrap(), &schema.borrow(), &mut pack_file.borrow_mut()) {
                Ok(tree_path) => tree_path,
                Err(error) => {
                    show_dialog(&app_ui.window, false, error.cause());
                    return Inhibit(false)
                }
            };

            // Set the mod as "Modified".
            set_modified(true, &app_ui.window, &mut pack_file.borrow_mut());

            // Add and update each path to the TreeView.
            for path in tree_paths.1 {

                // If it's one of the paths we removed and read it, skip it.
                if !tree_paths.0.contains(&path) {
                    update_treeview(
                        &app_ui.folder_tree_store,
                        &pack_file.borrow(),
                        &app_ui.folder_tree_selection,
                        TreeViewOperation::Add(path.to_vec()),
                        &TreePathType::None,
                    );
                }
            }

            // Destroy the "Settings Window".
            window.destroy();

            // Re-enable the main window.
            app_ui.window.set_sensitive(true);

            Inhibit(false)
        }
    ));

    // When we press the "Cancel" button, we close the window.
    cancel_button.connect_button_release_event(clone!(
        window,
        app_ui => move |_,_| {

            // Destroy the "Settings Window".
            window.destroy();

            // Re-enable the main window.
            app_ui.window.set_sensitive(true);

            Inhibit(false)
        }
    ));

    // When we close the window.
    window.connect_delete_event(clone!(
        app_ui => move |window,_| {

            // Destroy the "Settings Window".
            window.destroy();

            // Re-enable the main window.
            app_ui.window.set_sensitive(true);

            Inhibit(false)
        }
    ));
}

//----------------------------------------------------------------------------//
//              Utility functions (helpers and stuff like that)
//----------------------------------------------------------------------------//



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
        title
    );

    // Set the title and secondary text.
    dialog.set_title(title);
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
    status_bar.push(status_bar.get_context_id("Yekaterina"), &message.to_string());
}



/// This function cleans the accelerators and actions created by the PackedFile Views, so they can be
/// reused in another View.
pub fn remove_temporal_accelerators(application: &Application) {

    // Remove stuff of Loc View.
    application.set_accels_for_action("packedfile_loc_add_rows", &[]);
    application.set_accels_for_action("packedfile_loc_delete_rows", &[]);
    application.set_accels_for_action("packedfile_loc_copy_cell", &[]);
    application.set_accels_for_action("packedfile_loc_paste_cell", &[]);
    application.set_accels_for_action("packedfile_loc_copy_rows", &[]);
    application.set_accels_for_action("packedfile_loc_paste_rows", &[]);
    application.set_accels_for_action("packedfile_loc_copy_columns", &[]);
    application.set_accels_for_action("packedfile_loc_paste_columns", &[]);
    application.set_accels_for_action("packedfile_loc_import_tsv", &[]);
    application.set_accels_for_action("packedfile_loc_export_tsv", &[]);
    application.remove_action("packedfile_loc_add_rows");
    application.remove_action("packedfile_loc_delete_rows");
    application.remove_action("packedfile_loc_copy_cell");
    application.remove_action("packedfile_loc_paste_cell");
    application.remove_action("packedfile_loc_copy_rows");
    application.remove_action("packedfile_loc_paste_rows");
    application.remove_action("packedfile_loc_copy_columns");
    application.remove_action("packedfile_loc_paste_columns");
    application.remove_action("packedfile_loc_import_tsv");
    application.remove_action("packedfile_loc_export_tsv");

    // Remove stuff of DB View.
    application.set_accels_for_action("packedfile_db_add_rows", &[]);
    application.set_accels_for_action("packedfile_db_delete_rows", &[]);
    application.set_accels_for_action("packedfile_db_copy_cell", &[]);
    application.set_accels_for_action("packedfile_db_paste_cell", &[]);
    application.set_accels_for_action("packedfile_db_clone_rows", &[]);
    application.set_accels_for_action("packedfile_db_copy_rows", &[]);
    application.set_accels_for_action("packedfile_db_paste_rows", &[]);
    application.set_accels_for_action("packedfile_db_copy_columns", &[]);
    application.set_accels_for_action("packedfile_db_paste_columns", &[]);
    application.set_accels_for_action("packedfile_db_import_tsv", &[]);
    application.set_accels_for_action("packedfile_db_export_tsv", &[]);
    application.remove_action("packedfile_db_add_rows");
    application.remove_action("packedfile_db_delete_rows");
    application.remove_action("packedfile_db_copy_cell");
    application.remove_action("packedfile_db_paste_cell");
    application.remove_action("packedfile_db_clone_rows");
    application.remove_action("packedfile_db_copy_rows");
    application.remove_action("packedfile_db_paste_rows");
    application.remove_action("packedfile_db_copy_columns");
    application.remove_action("packedfile_db_paste_columns");
    application.remove_action("packedfile_db_import_tsv");
    application.remove_action("packedfile_db_export_tsv");

    // Remove stuff of DB decoder View.
    application.set_accels_for_action("move_row_up", &[]);
    application.set_accels_for_action("move_row_down", &[]);
    application.set_accels_for_action("delete_row", &[]);
    application.remove_action("move_row_up");
    application.remove_action("move_row_down");
    application.remove_action("delete_row");
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
*/

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
    //Add(Vec<String>),
    //AddFromPackFile(Vec<String>, Vec<String>, Vec<Vec<String>>),
    Delete(TreePathType),
    Rename(TreePathType, String),
}

/// This function shows a "Success" or "Error" Dialog with some text. For notification of success and
/// high importance errors.
/// It requires:
/// - parent_window: a reference to the `Window` that'll act as "parent" of the dialog.
/// - is_success: true for "Success" Dialog, false for "Error" Dialog.
/// - text: something that implements the trait "Display", so we want to put in the dialog window.
pub fn show_dialog<T: Display>(
    app_ui: &AppUI,
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
        app_ui.window as *mut Widget,
    )); }

    // Run the dialog.
    dialog.exec();
}

/// This function sets the currently open PackFile as "modified" or unmodified, both in the PackFile
/// and in the title bar, depending on the value of the "is_modified" boolean.
pub fn set_modified(
    is_modified: bool,
    app_ui: &AppUI,
) -> bool {

    // If the PackFile is modified...
    if is_modified {

        // Change the title of the Main Window.
        unsafe { app_ui.window.as_mut().unwrap().set_window_title(&QString::from_std_str("Rusted PackFile Manager (modified)")); }

        // And return true.
        true
    }

    // If it's not modified...
    else {

        // Change the title of the Main Window.
        unsafe { app_ui.window.as_mut().unwrap().set_window_title(&QString::from_std_str("Rusted PackFile Manager")); }

        // And return false.
        false
    }
}

/// This function delete whatever it's in the right side of the screen.
pub fn purge_them_all(app_ui: &AppUI) {
    unsafe {
        for _ in 0..app_ui.packed_file_layout.as_mut().unwrap().count() {
            let child = app_ui.packed_file_layout.as_mut().unwrap().take_at(0);
            child.as_mut().unwrap().widget().as_mut().unwrap().close();
            app_ui.packed_file_layout.as_mut().unwrap().remove_item(child);
        }
    }
}

/// This function shows a Message in the specified Grid.
pub fn display_help_tips(app_ui: &AppUI) {

    let label = Label::new(&QString::from_std_str("Welcome to Rusted PackFile Manager! Here you have some tips on how to use it:
        - You can see all the hotkeys in \"About/Shortcuts\".
        - To search in a DB Table or Loc PackedFile, hit \"Ctrl + F\" and write.
        - You can open a PackFile by dragging it to the big PackFile Tree View.
        - To patch an Attila model to work in Warhammer, select it and press \"Patch to Warhammer 1&2\".
        - You can insta-patch your siege maps (if you're a mapper) with the \"Patch SiegeAI\" feature from the \"Special Stuff\" menu.")).into_raw();

    unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((label as *mut Widget, 0, 0, 1, 1)); }
}

/// This function shows a message asking for confirmation. For use in operations that implies unsaved
/// data loss. is_modified = true for when you can lose unsaved changes, is_delete_my_mod = true for
/// the deletion warning of MyMods.
pub fn are_you_sure(
    is_modified: &Rc<RefCell<bool>>,
    is_delete_my_mod: bool
) -> bool {

    // If the mod has been modified...
    if *is_modified.borrow() {

        // Create the dialog.
        let mut dialog = MessageBox::new((
            &QString::from_std_str("Rusted PackFile Manager"),
            &QString::from_std_str("There are some changes yet to be saved.\nAre you sure?"),
            Icon::Warning,
            65536, // No
            16384, // Yes
            1, // By default, select yes.)
        ));

        // Run the dialog and get the response. Yes => 3, No => 4.
        if dialog.exec() == 3 { true } else { false }
    }

    // If we are going to delete a MyMod...
    else if is_delete_my_mod {

        // Create the dialog.
        let mut dialog = MessageBox::new((
            &QString::from_std_str("Rusted PackFile Manager"),
            &QString::from_std_str("You are about to delete this MyMod from your disk.\nThere is no way to recover it after that.\nAre you sure?"),
            Icon::Warning,
            65536, // No
            16384, // Yes
            1, // By default, select yes.)
        ));

        // Run the dialog and get the response. Yes => 3, No => 4.
        if dialog.exec() == 3 { true } else { false }
    }

    // Otherwise, we allow the change directly.
    else { true }
}

/// This function is used to get the complete Path of a Selected Item in the TreeView.
/// I'm sure there are other ways to do it, but the TreeView has proven to be a mystery
/// BEYOND MY COMPREHENSION, so we use this for now.
/// It requires:
/// - folder_tree_selection: &TreeSelection of the place of the TreeView we want to know his TreePath.
/// - include_packfile: bool. True if we want the TreePath to include the PackFile's name.
pub fn get_path_from_selection(
    app_ui: &AppUI,
    include_packfile: bool
) -> Vec<String>{

    // Create the vector to hold the Path.
    let mut path: Vec<String> = vec![];

    // Get the selection of the TreeView.
    let selection_model;
    let mut selection;
    unsafe { selection_model = app_ui.folder_tree_view.as_mut().unwrap().selection_model(); }
    unsafe { selection = selection_model.as_mut().unwrap().selected_indexes(); }

    // Get the selected cell.
    let mut item = selection.take_at(0);
    let mut parent;

    // Loop until we reach the root index.
    loop {

        // Get his data.
        let name;
        unsafe { name = QString::to_std_string(&app_ui.folder_tree_model.as_mut().unwrap().data(&item).to_string()); }

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

/// This function is used to get the complete Path of a Selected Item in the TreeView.
/// I'm sure there are other ways to do it, but the TreeView has proven to be a mystery
/// BEYOND MY COMPREHENSION, so we use this for now.
/// It requires:
/// - folder_tree_selection: &TreeSelection of the place of the TreeView we want to know his TreePath.
/// - include_packfile: bool. True if we want the TreePath to include the PackFile's name.
pub fn get_path_from_item_selection(
    app_ui: &AppUI,
    item: &ItemSelection,
    include_packfile: bool
) -> Vec<String>{

    // Create the vector to hold the Path.
    let mut path: Vec<String> = vec![];

    // Get the selection of the TreeView.
    let mut selection = item.indexes();

    // Get the selected cell.
    let mut item = selection.take_at(0);
    let mut parent;

    // Loop until we reach the root index.
    loop {

        // Get his data.
        let name;
        unsafe { name = QString::to_std_string(&app_ui.folder_tree_model.as_mut().unwrap().data(&item).to_string()); }

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

/// This function updates the provided `TreeView`, depending on the operation we want to do.
/// It requires:
/// - folder_tree_store: `&TreeStore` that the `TreeView` uses.
/// - mut pack_file_decoded: `&mut PackFile` we have opened, to get the data for the `TreeView`.
/// - folder_tree_selection: `&TreeSelection`, if there is something selected when we run this.
/// - operation: the `TreeViewOperation` we want to realise.
/// - type: the type of whatever is selected.
pub fn update_treeview(
    app_ui: &AppUI,
    pack_file_data: (&str, Vec<Vec<String>>), // (packfile_name, list of packfiles)
    operation: TreeViewOperation,
) {

    // We act depending on the operation requested.
    match operation {

        // If we want to build a new TreeView...
        TreeViewOperation::Build => {

            // First, we clean the TreeStore and whatever was created in the TreeView.
            unsafe { app_ui.folder_tree_model.as_mut().unwrap().clear(); }

            // Second, we set as the big_parent, the base for the folders of the TreeView, a fake folder
            // with the name of the PackFile. All big things start with a lie.
            let mut big_parent = StandardItem::new(&QString::from_std_str(pack_file_data.0));
            unsafe { app_ui.folder_tree_model.as_mut().unwrap().append_row_unsafe(big_parent.into_raw()); }

            // Third, we get all the paths of the PackedFiles inside the Packfile in a Vector.
            let mut sorted_path_list = pack_file_data.1;

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
            for path in &sorted_path_list {

                // First, we reset the parent to the big_parent (the PackFile).
                let mut parent;
                unsafe { parent = app_ui.folder_tree_model.as_ref().unwrap().item(0); }

                // Then, we form the path ("parent -> child" style path) to add to the TreeStore.
                for name in path.iter() {

                    // If it's the last string in the file path, it's a file, so we add it to the TreeStore.
                    if name == path.last().unwrap() {

                        // Add the file to the TreeView.
                        let mut file = StandardItem::new(&QString::from_std_str(name));
                        unsafe { parent.as_mut().unwrap().append_row_unsafe(file.into_raw()); }
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

                                    // Add the file to the TreeView.
                                    let mut folder = StandardItem::new(&QString::from_std_str(name));
                                    parent.as_mut().unwrap().append_row_unsafe(folder.into_raw());

                                    // This is our parent now.
                                    let index = parent.as_ref().unwrap().row_count() - 1;
                                    parent = parent.as_mut().unwrap().child(index);
                                }
                            }

                            // If our current parent doesn't have anything, just add it.
                            else {

                                // Add the file to the TreeView.
                                let mut folder = StandardItem::new(&QString::from_std_str(name));
                                parent.as_mut().unwrap().append_row_unsafe(folder.into_raw());

                                // This is our parent now.
                                let index = parent.as_ref().unwrap().row_count() - 1;
                                parent = parent.as_mut().unwrap().child(index);
                            }
                        }
                    }
                }
            }
        },
/*
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
                                let current_path = get_path_from_tree_iter(&tree_iter_test, folder_tree_store, true);
                                let current_type = get_type_of_selected_tree_path(&current_path, pack_file_decoded);

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
                                tree_iter = folder_tree_store.append(&tree_iter);

                                // Set his value.
                                folder_tree_store.set_value(&tree_iter, 0, &path[index].to_value());

                                // Sort properly the `TreeStore` to show the renamed file in his proper place.
                                sort_tree_view(folder_tree_store, pack_file_decoded, &new_type, &tree_iter);

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
                        tree_iter = folder_tree_store.append(&tree_iter);

                        // Set his value.
                        folder_tree_store.set_value(&tree_iter, 0, &path[index].to_value());

                        // Sort properly the `TreeStore` to show the renamed file in his proper place.
                        sort_tree_view(folder_tree_store, pack_file_decoded, &new_type, &tree_iter);

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
            if !source_prefix.is_empty() {

                // Take our the last folder.
                source_prefix.pop();

            }

            // For each file...
            for file in &new_files_list {

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
                    &TreePathType::File((vec![String::new()],1)),
                );
            }
        },
        */
        // If we want to delete something from the `TreeView`...
        TreeViewOperation::Delete(path_type) => {

            // Then we see what type the selected thing is.
            match path_type {

                // If it's a PackedFile or a Folder...
                TreePathType::File(_) | TreePathType::Folder(_) => {

                    // Get whatever is selected from the TreeView.
                    let packfile;
                    let selection_model;
                    let mut selection;
                    unsafe { selection_model = app_ui.folder_tree_view.as_mut().unwrap().selection_model(); }
                    unsafe { selection = selection_model.as_mut().unwrap().selected_indexes(); }
                    unsafe { packfile = app_ui.folder_tree_model.as_ref().unwrap().item(0); }
                    let mut item = selection.take_at(0);
                    let mut parent;

                    // Begin the endless cycle of war and dead.
                    loop {

                        // Get the parent of the item.
                        parent = item.parent();

                        // Kill the item in a cruel way.
                        unsafe { app_ui.folder_tree_model.as_mut().unwrap().remove_row((item.row(), &parent));}

                        // Check if the parent still has children.
                        let has_children;
                        let packfile_has_children;
                        unsafe { has_children = app_ui.folder_tree_model.as_mut().unwrap().has_children(&parent); }
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
                    let selection_model;
                    let mut selection;
                    unsafe { selection_model = app_ui.folder_tree_view.as_mut().unwrap().selection_model(); }
                    unsafe { selection = selection_model.as_mut().unwrap().selected_indexes(); }
                    let item = selection.at(0);
                    let name;
                    unsafe { name = app_ui.folder_tree_model.as_mut().unwrap().data(item).to_string(); }

                    // Clear the TreeModel.
                    unsafe { app_ui.folder_tree_model.as_mut().unwrap().clear(); }

                    // Then we add the PackFile to it. This effectively deletes all the PackedFiles in the PackFile.
                    let mut pack_file = StandardItem::new(&name);
                    unsafe { app_ui.folder_tree_model.as_mut().unwrap().append_row_unsafe(pack_file.into_raw()); }
                },

                // If we don't have anything selected, we do nothing.
                TreePathType::None => {},
            }
        },

        // If we want to rename something...
        TreeViewOperation::Rename(path_type, new_name) => {

            // Get the selection model.
            let selection_model;
            unsafe { selection_model = app_ui.folder_tree_view.as_mut().unwrap().selection_model(); }

            // Get the selected cell.
            let selection;
            unsafe { selection = selection_model.as_mut().unwrap().selected_indexes(); }
            let selection = selection.at(0);

            // Put the new name in a variant.
            let variant = Variant::new0(&QString::from_std_str(&new_name));

            // Change the old data with the new one.
            unsafe { app_ui.folder_tree_model.as_mut().unwrap().set_data((selection, &variant)); }

            // TODO: Fix this function, so when renaming stuff, it also get's sorted.
            // Sort properly the `TreeStore` to show the renamed file in his proper place.
            //sort_tree_view(folder_tree_store, pack_file_decoded, selection_type, &tree_iter)
        },
    }
}
/*
/// This function is meant to sort newly added items in a `TreeView`. Pls note that the provided
/// `&TreeIter` MUST BE VALID. Otherwise, this can CTD the entire program.
fn sort_tree_view(
    folder_tree_store: &TreeStore,
    pack_file_decoded: &PackFile,
    selection_type: &TreePathType,
    tree_iter: &TreeIter
) {

    // Get the previous and next `TreeIter`s.
    let tree_iter_previous = tree_iter.clone();
    let tree_iter_next = tree_iter.clone();

    let iter_previous_exists = folder_tree_store.iter_previous(&tree_iter_previous);
    let iter_next_exists = folder_tree_store.iter_next(&tree_iter_next);

    // If the previous iter is valid, get their path and their type.
    let previous_type = if iter_previous_exists {

        let path_previous = get_path_from_tree_iter(&tree_iter_previous, folder_tree_store, true);
        get_type_of_selected_tree_path(&path_previous, pack_file_decoded)
    }

    // Otherwise, return the type as `None`.
    else { TreePathType::None };

    // If the next iter is valid, get their path and their type.
    let next_type = if iter_next_exists {

        let path_next = get_path_from_tree_iter(&tree_iter_next, folder_tree_store, true);
        get_type_of_selected_tree_path(&path_next, pack_file_decoded)
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
        if selection_type == &TreePathType::Folder(vec![String::new()]) { true } else { false }
    }

    // If the two around it are the same type, compare them and decide.
    else {

        // Get the previous, current and next texts.
        let previous_name: String = folder_tree_store.get_value(&tree_iter_previous, 0).get().unwrap();
        let current_name: String = folder_tree_store.get_value(tree_iter, 0).get().unwrap();
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
            let path_second = get_path_from_tree_iter(&tree_iter_second, folder_tree_store, true);

            // Get the type of both `TreeIter`.
            let second_type = get_type_of_selected_tree_path(&path_second, pack_file_decoded);

            // If we have something of the same type than our `TreeIter`...
            if second_type == *selection_type {

                // Get the other `TreeIter`s text.
                let second_name: String = folder_tree_store.get_value(&tree_iter_second, 0).get().unwrap();
                let current_name: String = folder_tree_store.get_value(tree_iter, 0).get().unwrap();

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
                        folder_tree_store.swap(tree_iter, &tree_iter_second);
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
                        folder_tree_store.swap(tree_iter, &tree_iter_second);
                    }

                }
            }

            // If the top one is a File and the bottom one a Folder, it's an special situation. Just swap them.
            else if selection_type == &TreePathType::Folder(vec![String::new()]) && second_type == TreePathType::File((vec![String::new()], 1)) {
                folder_tree_store.swap(tree_iter, &tree_iter_second);
            }

            // If the type is different and it's not an special situation, we can't move anymore.
            else { break; }
        }

        // If the `TreeIter` is invalid, we can't move anymore.
        else { break; }
    }
}
*/
