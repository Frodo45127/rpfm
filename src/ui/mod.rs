// In this file are all the helper functions used by the UI (mainly GTK here)
extern crate num;
extern crate url;
extern crate failure;
extern crate qt_widgets;
extern crate qt_gui;
extern crate qt_core;
extern crate cpp_utils;
extern crate serde_json;

use qt_widgets::action::Action;
use qt_widgets::push_button::PushButton;
use failure::Error;
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
use qt_widgets::layout::Layout;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::dialog::Dialog;
use qt_core::item_selection::ItemSelection;
use qt_core::flags::Flags;
use qt_gui::icon;
use qt_widgets::size_policy::SizePolicy;
use qt_widgets::size_policy::Policy;
use qt_core::slots::SlotItemSelectionRefItemSelectionRef;

use qt_gui::standard_item_model::StandardItemModel;
use qt_gui::standard_item::StandardItem;
use qt_core::item_selection_model::ItemSelectionModel;
use qt_core::model_index::ModelIndex;
use qt_core::event_loop::EventLoop;
use qt_core::connection::Signal;
use qt_core::variant::Variant;
use qt_core::slots::SlotBool;
use qt_core::slots::SlotNoArgs;
use qt_core::object::Object;
use cpp_utils::{CppBox, StaticCast, DynamicCast};
use qt_core::qt::SortOrder;

use std::thread;
use std::time::Duration;
use std::sync::mpsc::{channel, Sender, Receiver};
use url::Url;
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::fmt::Display;

use QString;
use AppUI;
use common::*;
use packedfile::*;
use packedfile::db::*;
use packedfile::db::schemas::Schema;
use packedfile::loc::*;


pub mod packedfile_db;
pub mod packedfile_loc;
pub mod packedfile_text;
pub mod packedfile_image;
pub mod settings;
pub mod updater;
/*




use packfile::packfile::PackFile;
use packfile::packfile::PackedFile;

pub mod packedfile_rigidmodel;


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

*/

//----------------------------------------------------------------------------//
//             UI Creation functions (to build the UI on start)
//----------------------------------------------------------------------------//

/// This struct will hold all the MyMod-related stuff we have to recreate from time to time.
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

/// This struct will hold all the "Add From PackFile" stuff we have to keep alive.
#[derive(Copy, Clone)]
pub struct AddFromPackFileStuff {
    pub tree_view: *mut TreeView,
    pub tree_model: *mut StandardItemModel,
    pub exit_button: *mut PushButton,
    pub copy_button: *mut PushButton,
}

/// This struct holds all the Slots related to the "Add from PackFile" View, as otherwise they'll
/// die before we press their buttons and do nothing.
pub struct AddFromPackFileSlots {
    pub copy_check: SlotItemSelectionRefItemSelectionRef<'static>,
    pub copy: SlotNoArgs<'static>,
    pub exit: SlotNoArgs<'static>,
}

/// Implementation of "AddFromPackFileStuff".
impl AddFromPackFileStuff {

    /// This function creates a new "Add From PackFile" struct and returns it. This is just for
    /// initialization when starting the program.
    pub fn new() -> (AddFromPackFileStuff, AddFromPackFileSlots) {

        // Create the stuff.
        let tree_view = TreeView::new();
        let tree_model = StandardItemModel::new(());
        let exit_button = PushButton::new(&QString::from_std_str("Exit 'Add from Packfile' Mode"));
        let copy_button = PushButton::new(&QString::from_std_str("<="));

        // Create some dummy slots.
        let slots = AddFromPackFileSlots {
            copy_check: SlotItemSelectionRefItemSelectionRef::new(|_,_| {}),
            copy: SlotNoArgs::new(|| {}),
            exit: SlotNoArgs::new(|| {}),
        };

        // Create the new struct and return it.
        (AddFromPackFileStuff {
            tree_view: tree_view.into_raw(),
            tree_model: tree_model.into_raw(),
            exit_button: exit_button.into_raw(),
            copy_button: copy_button.into_raw(),
        }, slots)
    }

    /// This function creates a new "Add From PackFile" struct and returns it.
    pub fn new_with_grid(
        rpfm_path: PathBuf,
        sender_qt: Sender<&'static str>,
        sender_qt_data: &Sender<Result<Vec<u8>, Error>>,
        receiver_qt: &Rc<RefCell<Receiver<Result<Vec<u8>, Error>>>>,
        app_ui: AppUI,
        is_folder_tree_view_locked: &Rc<RefCell<bool>>,
        is_modified: &Rc<RefCell<bool>>,
        is_packedfile_opened: &Rc<RefCell<bool>>
    ) -> (AddFromPackFileStuff, AddFromPackFileSlots) {

        // Create the stuff.
        let mut tree_view = TreeView::new();
        let mut tree_model = StandardItemModel::new(());
        let mut exit_button = PushButton::new(&QString::from_std_str("Exit 'Add from Packfile' Mode"));
        let mut copy_button = PushButton::new(&QString::from_std_str("<="));

        // Configure it.
        unsafe { tree_view.set_model(tree_model.static_cast_mut()); }
        tree_view.set_header_hidden(true);
        copy_button.set_size_policy((Policy::Maximum, Policy::Expanding));

        // Add all the stuff to the Grid.
        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((exit_button.static_cast_mut() as *mut Widget, 0, 0, 1, 2)); }
        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((copy_button.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((tree_view.static_cast_mut() as *mut Widget, 1, 1, 1, 1)); }

        // Create the new struct with the widgets.
        let stuff = AddFromPackFileStuff {
            tree_view: tree_view.into_raw(),
            tree_model: tree_model.into_raw(),
            exit_button: exit_button.into_raw(),
            copy_button: copy_button.into_raw(),
        };

        // Create the slots for the stuff we need.
        let slots = AddFromPackFileSlots {
            copy_check: SlotItemSelectionRefItemSelectionRef::new(clone!(
                sender_qt,
                sender_qt_data,
                receiver_qt => move |selection,_| {

                    // Get the path of the selected item in the main TreeView.
                    let path = get_path_from_item_selection(app_ui.folder_tree_model, &selection, true);

                    // Send the Path to the Background Thread, and get the type of the item.
                    sender_qt.send("get_type_of_path").unwrap();
                    sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();
                    let response = receiver_qt.borrow().recv().unwrap().unwrap();
                    let item_type: TreePathType = serde_json::from_slice(&response).unwrap();

                    // Depending on the type of the selected item, we enable or disable the copy button.
                    match item_type {
                        TreePathType::File(_) | TreePathType::None => unsafe { stuff.copy_button.as_mut().unwrap().set_enabled(false) }
                        _ => unsafe { stuff.copy_button.as_mut().unwrap().set_enabled(true) },
                    }
                }
            )),
            copy: SlotNoArgs::new(clone!(
                is_modified,
                sender_qt,
                sender_qt_data,
                receiver_qt => move || {

                    // Get the selections of both TreeViews.
                    let selection_source;
                    let selection_destination;
                    unsafe { selection_source = stuff.tree_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                    unsafe { selection_destination = app_ui.folder_tree_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }

                    // Get his source & destination paths.
                    let path_source = get_path_from_item_selection(stuff.tree_model, &selection_source, true);
                    let path_destination = get_path_from_item_selection(app_ui.folder_tree_model, &selection_destination, true);

                    // Ask the Background Thread to move the files, and send him the paths.
                    sender_qt.send("add_packedfile_from_packfile").unwrap();
                    sender_qt_data.send(serde_json::to_vec(&(path_source, path_destination)).map_err(From::from)).unwrap();

                    // Prepare the event loop, so we don't hang the UI while the background thread is working.
                    let mut event_loop = EventLoop::new();

                    // Disable the Main Window (so we can't do other stuff).
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                    // Until we receive a response from the worker thread...
                    loop {

                        // When we finally receive the data...
                        if let Ok(data) = receiver_qt.borrow().try_recv() {

                            // Check what the result of the process was.
                            match data {

                                // In case of success...
                                Ok(response) => {

                                    // Deserialize the response
                                    let paths: (Vec<String>, Vec<String>, Vec<Vec<String>>) = serde_json::from_slice(&response).unwrap();

                                    // Set the mod as "Modified".
                                    *is_modified.borrow_mut() = set_modified(true, &app_ui);

                                    // Update the TreeView.
                                    update_treeview(
                                        &rpfm_path,
                                        &sender_qt,
                                        &sender_qt_data,
                                        receiver_qt.clone(),
                                        app_ui.folder_tree_view,
                                        app_ui.folder_tree_model,
                                        TreeViewOperation::AddFromPackFile(paths.0, paths.1, paths.2),
                                    );
                                }

                                // In case of error, show the dialog with the error.
                                Err(error) => show_dialog(&app_ui, false, format!("Error while trying to add a PackedFile:\n\n{}", error.cause())),
                            }

                            // Stop the loop.
                            break;
                        }

                        // Keep the UI responsive.
                        event_loop.process_events(());

                        // Wait a bit to not saturate a CPU core.
                        thread::sleep(Duration::from_millis(50));
                    }

                    // Re-enable the Main Window.
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                }
            )),
            exit: SlotNoArgs::new(clone!(
                is_packedfile_opened,
                is_folder_tree_view_locked => move || {

                    // Reset the Secondary PackFile.
                    sender_qt.send("reset_packfile_extra").unwrap();

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
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().selection_model().as_ref().unwrap().signals().selection_changed().connect(&slots.copy_check); }
        unsafe { stuff.copy_button.as_ref().unwrap().signals().released().connect(&slots.copy); }
        unsafe { stuff.exit_button.as_ref().unwrap().signals().released().connect(&slots.exit); }

        // Return the stuff and slots needed for it to work.
        (stuff, slots)
    }
}

/// This function creates all the "New PackedFile" dialogs. It returns the type/name of the new file,
/// or None if the dialog is canceled or closed.
pub fn create_new_packed_file_dialog(
    app_ui: &AppUI,
    packed_file_type: PackedFileType
) -> Option<PackedFileType> {

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

    // Add all the widgets to the main grid.
    unsafe { main_grid.as_mut().unwrap().add_widget((new_packed_file_name_edit.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((create_button as *mut Widget, 0, 1, 1, 1)); }

    // And the Main Grid to the Dialog...
    unsafe { dialog.set_layout(main_grid as *mut Layout); }

    //-------------------------------------------------------------------------------------------//
    // Actions for the New PackedFile Dialog...
    //-------------------------------------------------------------------------------------------//

    // What happens when we hit the "Rename" button.
    unsafe { create_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

    // Show the Dialog and, if we hit the "Rename" button...
    if dialog.exec() == 1 {

        // Get the text from the LineEdit.
        let packed_file_name = new_packed_file_name_edit.text().to_std_string();

        // Depending on the PackedFile's Type, return the new name.
        match packed_file_type {
            PackedFileType::Loc(_) => Some(PackedFileType::Loc(packed_file_name)),
            PackedFileType::DB(_,_,_) => Some(PackedFileType::DB(packed_file_name, "".to_owned(), 0)),
            PackedFileType::Text(_) => Some(PackedFileType::Text(packed_file_name)),
        }
    }

    // Otherwise, return None.
    else { None }
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
    Build(bool),
    Add(Vec<Vec<String>>),
    AddFromPackFile(Vec<String>, Vec<String>, Vec<Vec<String>>),
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
pub fn purge_them_all(app_ui: &AppUI, is_packedfile_opened: &Rc<RefCell<bool>>) {
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
    model: *mut StandardItemModel,
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
        unsafe { name = QString::to_std_string(&model.as_mut().unwrap().data(&item).to_string()); }

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
        unsafe { name = QString::to_std_string(&model.as_mut().unwrap().data(&item).to_string()); }

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

/// This function is used to get the complete TreePath (path in a GTKTreeView) of an external file
/// or folder in a Vec<String> format. Needed to get the path for the TreeView and for encoding
/// the file in a PackFile.
/// It requires:
/// - file_path: &PathBuf of the external file.
/// - folder_tree_selection: &TreeSelection of the place of the TreeView where we want to add the file.
/// - is_file: bool. True if the &PathBuf is from a file, false if it's a folder.
pub fn get_path_from_pathbuf(
    app_ui: &AppUI,
    file_path: &PathBuf,
    is_file: bool
) -> Vec<Vec<String>> {

    // Create the vector to hold the Path.
    let mut paths: Vec<Vec<String>> = vec![];

    // If it's a single file, we get his name and push it to the tree_path vector.
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

    // Return the paths (from parent to children)
    paths
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

/// Enum `IconType`: This enum holds all the possible Icon Types we can have in the TreeView,
/// depending on the type of the PackedFiles.
enum IconType {

    // For normal PackFiles. True for editable, false for read-only.
    PackFile(bool),

    // For folders.
    Folder,

    // For files with no other Icon. Includes the path without the Packfile.
    File(Vec<String>),
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

/// This function updates the provided `TreeView`, depending on the operation we want to do.
/// It requires:
/// - folder_tree_store: `&TreeStore` that the `TreeView` uses.
/// - mut pack_file_decoded: `&mut PackFile` we have opened, to get the data for the `TreeView`.
/// - folder_tree_selection: `&TreeSelection`, if there is something selected when we run this.
/// - operation: the `TreeViewOperation` we want to realise.
/// - type: the type of whatever is selected.
pub fn update_treeview(
    rpfm_path: &PathBuf,
    sender_qt: &Sender<&str>,
    sender_qt_data: &Sender<Result<Vec<u8>, Error>>,
    receiver_qt: Rc<RefCell<Receiver<Result<Vec<u8>, Error>>>>,
    tree_view: *mut TreeView,
    model: *mut StandardItemModel,
    operation: TreeViewOperation,
) {

    // Get the Icons for the TreeView.
    let icons = Icons::new(&rpfm_path);

    // We act depending on the operation requested.
    match operation {

        // If we want to build a new TreeView...
        TreeViewOperation::Build(is_extra_packfile) => {

            // Depending on what PackFile we want to build the TreeView on, we ask for his data.
            if is_extra_packfile { sender_qt.send("get_packfile_extra_data_for_treeview").unwrap(); }
            else { sender_qt.send("get_packfile_data_for_treeview").unwrap(); }

            // Get the data of the PackFile (PackFile's name + List of files).
            let response = receiver_qt.borrow().recv().unwrap().unwrap();
            let pack_file_data: (&str, Vec<Vec<String>>) = serde_json::from_slice(&response).unwrap();

            // First, we clean the TreeStore and whatever was created in the TreeView.
            unsafe { model.as_mut().unwrap().clear(); }

            // Second, we set as the big_parent, the base for the folders of the TreeView, a fake folder
            // with the name of the PackFile. All big things start with a lie.
            let mut big_parent = StandardItem::new(&QString::from_std_str(pack_file_data.0)).into_raw();

            // Also, set it as not editable by the user. Otherwise will cause problems when renaming.
            unsafe { big_parent.as_mut().unwrap().set_editable(false); }

            // Add the Big Parent to the Model.
            unsafe { model.as_mut().unwrap().append_row_unsafe(big_parent); }

            // Give it an Icon.
            set_icon_to_item(big_parent, &icons, IconType::PackFile(is_extra_packfile));

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
                unsafe { parent = model.as_ref().unwrap().item(0); }

                // Then, we form the path ("parent -> child" style path) to add to the TreeStore.
                for name in path.iter() {

                    // If it's the last string in the file path, it's a file, so we add it to the TreeStore.
                    if name == path.last().unwrap() {

                        // Add the file to the TreeView.
                        let mut file = StandardItem::new(&QString::from_std_str(name)).into_raw();

                        // Also, set it as not editable by the user. Otherwise will cause problems when renaming.
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

                                    // Add the folder to the TreeView.
                                    let mut folder = StandardItem::new(&QString::from_std_str(name)).into_raw();

                                    // Also, set it as not editable by the user. Otherwise will cause problems when renaming.
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

                                // Add the folder to the TreeView.
                                let mut folder = StandardItem::new(&QString::from_std_str(name)).into_raw();

                                // Also, set it as not editable by the user. Otherwise will cause problems when renaming.
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

                        // Add the file to the TreeView.
                        let item = StandardItem::new(&QString::from_std_str(field)).into_raw();

                        // Also, set it as not editable by the user. Otherwise will cause problems when renaming.
                        unsafe { item.as_mut().unwrap().set_editable(false); }
                        unsafe { parent.as_mut().unwrap().append_row_unsafe(item); }

                        // Get the Path of the File.
                        let path = get_path_from_item(model, item, true);

                        // Send the Path to the Background Thread, and get the type of the item.
                        sender_qt.send("get_type_of_path").unwrap();
                        sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();
                        let response = receiver_qt.borrow().recv().unwrap().unwrap();
                        let item_type: TreePathType = serde_json::from_slice(&response).unwrap();

                        // Get the incomplete Path.
                        let mut incomplete_path = path.to_vec();
                        incomplete_path.reverse();
                        incomplete_path.pop();
                        incomplete_path.reverse();

                        // If it's a Folder...
                        if item_type == TreePathType::Folder(vec![String::new()]) {

                            // Give it a folder icon.
                            set_icon_to_item(item, &icons, IconType::Folder);
                        }

                        // Otherwise, give it an icon.
                        else { set_icon_to_item(item, &icons, IconType::File(incomplete_path)); }

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

                                    // Add the file to the TreeView.
                                    let mut folder = StandardItem::new(&QString::from_std_str(field)).into_raw();

                                    // Also, set it as not editable by the user. Otherwise will cause problems when renaming.
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

                                // Add the file to the TreeView.
                                let mut folder = StandardItem::new(&QString::from_std_str(field)).into_raw();

                                // Also, set it as not editable by the user. Otherwise will cause problems when renaming.
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

        // If we want to add a file/folder from another `TreeView`...
        TreeViewOperation::AddFromPackFile(mut source_prefix, destination_prefix, new_files_list) => {

            // If his path is something, take our the last folder.
            if !source_prefix.is_empty() { source_prefix.pop(); }

            // Create a Vector to store the final paths of the files.
            let mut final_paths_list = vec![];

            // For each file...
            for file in &new_files_list {

                // Filter his new path.
                let mut filtered_source_path = file[source_prefix.len()..].to_vec();
                let mut final_path = destination_prefix.to_vec();
                final_path.append(&mut filtered_source_path);

                // And add it to the list.
                final_paths_list.push(final_path);
            }

            // Update the TreeView with all the new Paths.
            update_treeview(
                &rpfm_path,
                &sender_qt,
                &sender_qt_data,
                receiver_qt.clone(),
                tree_view,
                model,
                TreeViewOperation::Add(final_paths_list),
            );
        },

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

                    // Get the name of the PackFile from the TreeView.
                    let selection_model;
                    let mut selection;
                    unsafe { selection_model = tree_view.as_mut().unwrap().selection_model(); }
                    unsafe { selection = selection_model.as_mut().unwrap().selected_indexes(); }
                    let item = selection.at(0);
                    let name;
                    unsafe { name = model.as_mut().unwrap().data(item).to_string(); }

                    // Clear the TreeModel.
                    unsafe { model.as_mut().unwrap().clear(); }

                    // Then we add the PackFile to it. This effectively deletes all the PackedFiles in the PackFile.
                    let mut pack_file = StandardItem::new(&name);
                    unsafe { model.as_mut().unwrap().append_row_unsafe(pack_file.into_raw()); }
                },

                // If we don't have anything selected, we do nothing.
                TreePathType::None => {},
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

            // If what we are renaming is not the PackFile, sort the item in the TreeView.
            if path_type != TreePathType::PackFile {

                // Get the item.
                let item;
                unsafe { item = model.as_mut().unwrap().item_from_index(selection); }

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
        },
    }

    // If we have altered the TreeView in ANY way, we need to recheck the empty folders list.
    sender_qt.send("update_empty_folders").unwrap();
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
    sender_qt: &Sender<&str>,
    sender_qt_data: &Sender<Result<Vec<u8>, Error>>,
    receiver_qt: Rc<RefCell<Receiver<Result<Vec<u8>, Error>>>>,
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
        sender_qt.send("get_type_of_path").unwrap();
        sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();
        let response = receiver_qt.borrow().recv().unwrap().unwrap();
        serde_json::from_slice(&response).unwrap()
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
        sender_qt.send("get_type_of_path").unwrap();
        sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();
        let response = receiver_qt.borrow().recv().unwrap().unwrap();
        serde_json::from_slice(&response).unwrap()
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
            sender_qt.send("get_type_of_path").unwrap();
            sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();
            let response = receiver_qt.borrow().recv().unwrap().unwrap();
            let item_sibling_type: TreePathType = serde_json::from_slice(&response).unwrap();

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
