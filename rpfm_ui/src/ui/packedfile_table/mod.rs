//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// This file contains all the common stuff between DB, the DB Decoder and Locs, 
// to reduce duplicated code. It also houses the DB Decoder, because thatś 
// related with the tables.

use qt_widgets::action::Action;
use qt_widgets::file_dialog::FileDialog;
use qt_widgets::header_view::ResizeMode;
use qt_widgets::menu::Menu;
use qt_widgets::label::Label;
use qt_widgets::slots::{SlotQtCorePointRef, SlotCIntQtCoreQtSortOrder};
use qt_widgets::table_view::TableView;
use qt_widgets::scroll_area::ScrollArea;
use qt_widgets::widget::Widget;

use qt_gui::cursor::Cursor;
use qt_gui::gui_application::GuiApplication;
use qt_gui::key_sequence::KeySequence;
use qt_gui::list::ListStandardItemMutPtr;
use qt_gui::slots::{SlotStandardItemMutPtr, SlotCIntCIntCInt};
use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::flags::Flags;
use qt_core::signal_blocker::SignalBlocker;
use qt_core::sort_filter_proxy_model::SortFilterProxyModel;
use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::connection::Signal;
use qt_core::variant::Variant;
use qt_core::item_selection::ItemSelection;
use qt_core::item_selection_model::SelectionFlag;
use qt_core::object::Object;
use qt_core::reg_exp::RegExp;
use qt_core::slots::{SlotBool, SlotCInt, SlotStringRef, SlotItemSelectionRefItemSelectionRef, SlotModelIndexRefModelIndexRefVectorVectorCIntRef};
use qt_core::string_list::StringList;
use qt_core::qt::{AlignmentFlag, CaseSensitivity, CheckState, ShortcutContext, SortOrder, GlobalColor, MatchFlag};

use regex::Regex;
use meval;

use std::collections::BTreeMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::TABLE_STATES_UI;
use crate::QString;
use crate::ui::*;
use rpfm_lib::packedfile::db::DB;
use rpfm_lib::packedfile::loc::Loc;
use crate::ui::qt_custom_stuff::*;
use crate::ui::table_state::*;
use crate::ui::packedfile_table::packedfile_table_undo::*;
use crate::ui::packedfile_table::packedfile_table_extras::*;

pub mod db_decoder;
pub mod packedfile_db;
pub mod packedfile_loc;
pub mod dependency_manager;
mod packedfile_table_extras;
mod packedfile_table_undo;

//----------------------------------------------------------------//
// Generic Enums and Structs for DB/LOC PackedFiles.
//----------------------------------------------------------------//

/// Enum `TableType`: used to distinguis between DB and Loc.
#[derive(Clone)]
pub enum TableType {
    DependencyManager(Vec<Vec<DecodedData>>),
    DB(DB),
    LOC(Loc),
}

/// Enum to know what operation was done while editing tables, so we can revert them with undo.
/// - Editing: Intended for any kind of item editing. Holds a Vec<((row, column), *mut item)>, so we can do this in batches.
/// - AddRows: Intended for when adding/inserting rows. It holds a list of positions where the rows where inserted.
/// - RemoveRows: Intended for when removing rows. It holds a list of positions where the rows where deleted and the deleted rows data, in consecutive batches.
/// - SmartDelete: Intended for when we are using the smart delete feature. This is a combination of list of edits and list of removed rows.
/// - RevertSmartDelete: Selfexplanatory. This is a combination of list of edits and list of adding rows.
/// - ImportTSV: It holds a copy of the entire table, before importing.
/// - Carolina: A Jack-of-all-Trades. It holds a Vec<TableOperations>, for those situations one is not enough.
#[derive(Clone)]
pub enum TableOperations {
    Editing(Vec<((i32, i32), *mut StandardItem)>),
    AddRows(Vec<i32>),
    RemoveRows((Vec<Vec<(i32, Vec<*mut StandardItem>)>>)),
    SmartDelete((Vec<((i32, i32), *mut StandardItem)>, Vec<Vec<(i32, Vec<*mut StandardItem>)>>)),
    RevertSmartDelete((Vec<((i32, i32), *mut StandardItem)>, Vec<i32>)),
    ImportTSV(Vec<Vec<DecodedData>>),
    Carolina(Vec<TableOperations>),
}

/// Struct `PackedFileTableView`: contains all the stuff we need to give to the program to show a
/// TableView with the data of a DB/LOC PackedFile, allowing us to manipulate it.
pub struct PackedFileTableView {
    pub slot_column_moved: SlotCIntCIntCInt<'static>,
    pub slot_sort_order_column_changed: SlotCIntQtCoreQtSortOrder<'static>,
    pub slot_undo: SlotNoArgs<'static>,
    pub slot_redo: SlotNoArgs<'static>,
    pub slot_undo_redo_enabler: SlotNoArgs<'static>,
    pub slot_context_menu: SlotQtCorePointRef<'static>,
    pub slot_context_menu_enabler: SlotItemSelectionRefItemSelectionRef<'static>,
    pub save_changes: SlotModelIndexRefModelIndexRefVectorVectorCIntRef<'static>,
    pub slot_item_changed: SlotStandardItemMutPtr<'static>,
    pub slot_row_filter_change_text: SlotStringRef<'static>,
    pub slot_row_filter_change_column: SlotCInt<'static>,
    pub slot_row_filter_change_case_sensitive: SlotBool<'static>,
    pub slot_context_menu_add: SlotBool<'static>,
    pub slot_context_menu_insert: SlotBool<'static>,
    pub slot_context_menu_delete: SlotBool<'static>,
    pub slot_context_menu_apply_maths_to_selection: SlotBool<'static>,
    pub slot_context_menu_rewrite_selection: SlotBool<'static>,
    pub slot_context_menu_clone: SlotBool<'static>,
    pub slot_context_menu_clone_and_append: SlotBool<'static>,
    pub slot_context_menu_copy: SlotBool<'static>,
    pub slot_context_menu_copy_as_lua_table: SlotBool<'static>,
    pub slot_context_menu_paste: SlotBool<'static>,
    pub slot_context_menu_paste_as_new_lines: SlotBool<'static>,
    pub slot_context_menu_paste_to_fill_selection: SlotBool<'static>,
    pub slot_context_menu_selection_invert: SlotBool<'static>,
    pub slot_context_menu_search: SlotBool<'static>,
    pub slot_context_menu_sidebar: SlotBool<'static>,
    pub slot_context_menu_import: SlotBool<'static>,
    pub slot_context_menu_export: SlotBool<'static>,
    pub slot_smart_delete: SlotBool<'static>,
    pub slots_hide_show_column: Vec<SlotCInt<'static>>,
    pub slots_freeze_unfreeze_column: Vec<SlotCInt<'static>>,

    pub slot_update_search_stuff: SlotNoArgs<'static>,
    pub slot_search: SlotNoArgs<'static>,
    pub slot_prev_match: SlotNoArgs<'static>,
    pub slot_next_match: SlotNoArgs<'static>,
    pub slot_close_search: SlotNoArgs<'static>,
    pub slot_replace_current: SlotNoArgs<'static>,
    pub slot_replace_all: SlotNoArgs<'static>,

    // From here there is just stuff we need for the Table to work, not UI stuff.
    // pub undo_lock: Rc<RefCell<bool>>,
}

//----------------------------------------------------------------//
// Implementations of `TableOperation`.
//----------------------------------------------------------------//

/// Debug implementation of TableOperations, so we can at least guess what is in the history.
impl Debug for TableOperations {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TableOperations::Editing(data) => write!(f, "Cell/s edited, starting in row {}, column {}.", (data[0].0).0, (data[0].0).1),
            TableOperations::AddRows(data) => write!(f, "Row/s added in position/s {}.", data.iter().map(|x| format!("{}, ", x)).collect::<String>()),
            TableOperations::RemoveRows(data) => write!(f, "Row/s removed in {} batches.", data.len()),
            TableOperations::SmartDelete(_) => write!(f, "Smart deletion."),
            TableOperations::RevertSmartDelete(_) => write!(f, "Reverted Smart deletion."),
            TableOperations::ImportTSV(_) => write!(f, "Imported TSV file."),
            TableOperations::Carolina(_) => write!(f, "Carolina, trátame bien, no te rías de mi, no me arranques la piel."),
        }
    }
}

//----------------------------------------------------------------//
// Implementation of `PackedFileTableView`.
//----------------------------------------------------------------//

/// Implementation of PackedFileDBTreeView.
impl PackedFileTableView {

    /// This function creates a new Table with the PackedFile's View as father and returns a
    /// `PackedFileTableView` with all his data. This is the generic function for DB and LOCs.
    /// ANYTHING specific goes before or after this.
    pub fn create_table_view(
        sender_qt: &Sender<Commands>,
        sender_qt_data: &Sender<Data>,
        receiver_qt: &Rc<RefCell<Receiver<Data>>>,
        app_ui: &AppUI,
        layout: *mut GridLayout,
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        global_search_explicit_paths: &Rc<RefCell<Vec<Vec<String>>>>,
        update_global_search_stuff: *mut Action,
        table_state_data: &Rc<RefCell<BTreeMap<Vec<String>, TableStateData>>>,
        table_definition: &Rc<Definition>,
        enable_header_popups: Option<String>,
        table_type: &Rc<RefCell<TableType>>,
    ) -> Result<Self> {

        // Get the entire dependency data for this table.
        sender_qt.send(Commands::DecodeDependencyDB).unwrap();
        sender_qt_data.send(Data::Definition((&**table_definition).clone())).unwrap();
        let dependency_data: Rc<BTreeMap<i32, Vec<String>>> = Rc::new(match check_message_validity_recv2(&receiver_qt) { 
            Data::BTreeMapI32VecString(data) => data,
            Data::Error(_) => BTreeMap::new(),
            _ => panic!(THREADS_MESSAGE_ERROR), 
        });
        
        // Create the "Undo" stuff needed for the Undo/Redo functions to work.
        let undo_lock = Rc::new(RefCell::new(false));
        let undo_redo_enabler = Action::new(()).into_raw();
        if table_state_data.borrow().get(&*packed_file_path.borrow()).is_none() {
            let _y = table_state_data.borrow_mut().insert(packed_file_path.borrow().to_vec(), TableStateData::new_empty());
        }

        // Create the "Paste Lock", so we don't save in every freaking edit.
        let save_lock = Rc::new(RefCell::new(false));

        // Prepare the model and the filter..
        let filter_model = SortFilterProxyModel::new().into_raw();
        let model = StandardItemModel::new(()).into_raw();
        unsafe { filter_model.as_mut().unwrap().set_source_model(model as *mut AbstractItemModel); }
        
        // Prepare the TableViews.
        let table_view_frozen = TableView::new().into_raw();
        let table_view = unsafe { new_tableview_frozen(filter_model as *mut AbstractItemModel, table_view_frozen) };

        // Make the last column fill all the available space, if the setting says so.
        if SETTINGS.lock().unwrap().settings_bool["extend_last_column_on_tables"] { 
            unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }
        }

        // Create the filter's LineEdit.
        let row_filter_line_edit = LineEdit::new(()).into_raw();
        unsafe { row_filter_line_edit.as_mut().unwrap().set_placeholder_text(&QString::from_std_str("Type here to filter the rows in the table. Works with Regex too!")); }

        // Create the filter's column selector.
        // TODO: Make this follow the visual order of the columns, NOT THE LOGICAL ONE.
        let row_filter_column_selector = ComboBox::new().into_raw();
        let row_filter_column_list = StandardItemModel::new(()).into_raw();
        unsafe { row_filter_column_selector.as_mut().unwrap().set_model(row_filter_column_list as *mut AbstractItemModel); }
        for column in &table_definition.fields {
            let name = Self::clean_column_names(&column.field_name);
            unsafe { row_filter_column_selector.as_mut().unwrap().add_item(&QString::from_std_str(&name)); }
        }

        // Create the filter's "Case Sensitive" button.
        let row_filter_case_sensitive_button = PushButton::new(&QString::from_std_str("Case Sensitive")).into_raw();
        unsafe { row_filter_case_sensitive_button.as_mut().unwrap().set_checkable(true); }

        // Load the data to the Table. For some reason, if we do this after setting the titles of
        // the columns, the titles will be reseted to 1, 2, 3,... so we do this here.
        Self::load_data_to_table_view(table_view, model, &table_type.borrow(), table_definition, &dependency_data);

        // Add Table to the Grid.
        unsafe { layout.as_mut().unwrap().add_widget((table_view as *mut Widget, 0, 0, 1, 3)); }
        unsafe { layout.as_mut().unwrap().add_widget((row_filter_line_edit as *mut Widget, 2, 0, 1, 1)); }
        unsafe { layout.as_mut().unwrap().add_widget((row_filter_case_sensitive_button as *mut Widget, 2, 1, 1, 1)); }
        unsafe { layout.as_mut().unwrap().add_widget((row_filter_column_selector as *mut Widget, 2, 2, 1, 1)); }

        // Create the search and hide/show/freeze widgets.
        let search_widget = Widget::new().into_raw();
        let sidebar_widget = Widget::new().into_raw();
        let sidebar_scroll_area = ScrollArea::new().into_raw();
        let grid = create_grid_layout_unsafe(search_widget);
        let sidebar_grid = create_grid_layout_unsafe(sidebar_widget);
        unsafe { sidebar_scroll_area.as_mut().unwrap().set_widget(sidebar_widget); }
        unsafe { sidebar_scroll_area.as_mut().unwrap().set_widget_resizable(true); }
        unsafe { sidebar_scroll_area.as_mut().unwrap().horizontal_scroll_bar().as_mut().unwrap().set_enabled(false); }
        unsafe { sidebar_grid.as_mut().unwrap().set_contents_margins((4, 0, 4, 4)); }
        unsafe { sidebar_grid.as_mut().unwrap().set_spacing(4); }

        // Create the "Search" Grid and his internal widgets.
        let matches_label = Label::new(());
        let search_label = Label::new(&QString::from_std_str("Search Pattern:"));
        let replace_label = Label::new(&QString::from_std_str("Replace Pattern:"));
        let mut search_line_edit = LineEdit::new(());
        let mut replace_line_edit = LineEdit::new(());
        let mut prev_match_button = PushButton::new(&QString::from_std_str("Prev. Match"));
        let mut next_match_button = PushButton::new(&QString::from_std_str("Next Match"));
        let search_button = PushButton::new(&QString::from_std_str("Search"));
        let mut replace_current_button = PushButton::new(&QString::from_std_str("Replace Current"));
        let mut replace_all_button = PushButton::new(&QString::from_std_str("Replace All"));
        let close_button = PushButton::new(&QString::from_std_str("Close"));
        let mut column_selector = ComboBox::new();
        let column_list = StandardItemModel::new(());
        let mut case_sensitive_button = PushButton::new(&QString::from_std_str("Case Sensitive"));

        search_line_edit.set_placeholder_text(&QString::from_std_str("Type here what you want to search."));
        replace_line_edit.set_placeholder_text(&QString::from_std_str("If you want to replace the searched text with something, type the replacement here."));

        unsafe { column_selector.set_model(column_list.into_raw() as *mut AbstractItemModel); }
        column_selector.add_item(&QString::from_std_str("* (All Columns)"));
        for column in &table_definition.fields {
            column_selector.add_item(&QString::from_std_str(&Self::clean_column_names(&column.field_name)));
        }
        case_sensitive_button.set_checkable(true);

        prev_match_button.set_enabled(false);
        next_match_button.set_enabled(false);
        replace_current_button.set_enabled(false);
        replace_all_button.set_enabled(false);

        let matches_label = matches_label.into_raw();
        let search_line_edit = search_line_edit.into_raw();
        let replace_line_edit = replace_line_edit.into_raw();
        let column_selector = column_selector.into_raw();
        let case_sensitive_button = case_sensitive_button.into_raw();
        let prev_match_button = prev_match_button.into_raw();
        let next_match_button = next_match_button.into_raw();
        let search_button = search_button.into_raw();
        let replace_current_button = replace_current_button.into_raw();
        let replace_all_button = replace_all_button.into_raw();
        let close_button = close_button.into_raw();

        // Add all the widgets to the search grid.
        unsafe { grid.as_mut().unwrap().add_widget((search_label.into_raw() as *mut Widget, 0, 0, 1, 1)); }
        unsafe { grid.as_mut().unwrap().add_widget((search_line_edit as *mut Widget, 0, 1, 1, 1)); }
        unsafe { grid.as_mut().unwrap().add_widget((prev_match_button as *mut Widget, 0, 2, 1, 1)); }
        unsafe { grid.as_mut().unwrap().add_widget((next_match_button as *mut Widget, 0, 3, 1, 1)); }
        unsafe { grid.as_mut().unwrap().add_widget((replace_label.into_raw() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { grid.as_mut().unwrap().add_widget((replace_line_edit as *mut Widget, 1, 1, 1, 3)); }
        unsafe { grid.as_mut().unwrap().add_widget((search_button as *mut Widget, 0, 4, 1, 1)); }
        unsafe { grid.as_mut().unwrap().add_widget((replace_current_button as *mut Widget, 1, 4, 1, 1)); }
        unsafe { grid.as_mut().unwrap().add_widget((replace_all_button as *mut Widget, 2, 4, 1, 1)); }
        unsafe { grid.as_mut().unwrap().add_widget((close_button as *mut Widget, 2, 0, 1, 1)); }
        unsafe { grid.as_mut().unwrap().add_widget((matches_label as *mut Widget, 2, 1, 1, 1)); }
        unsafe { grid.as_mut().unwrap().add_widget((column_selector as *mut Widget, 2, 2, 1, 1)); }
        unsafe { grid.as_mut().unwrap().add_widget((case_sensitive_button as *mut Widget, 2, 3, 1, 1)); }

        // Add all the stuff to the main grid and hide the search widget.
        unsafe { layout.as_mut().unwrap().add_widget((sidebar_scroll_area as *mut Widget, 0, 3, 3, 1)); }
        unsafe { layout.as_mut().unwrap().add_widget((search_widget as *mut Widget, 1, 0, 1, 3)); }
        unsafe { layout.as_mut().unwrap().set_column_stretch(0, 10); }
        unsafe { search_widget.as_mut().unwrap().hide(); }
        unsafe { sidebar_scroll_area.as_mut().unwrap().hide(); }
        unsafe { sidebar_grid.as_mut().unwrap().set_row_stretch(999, 10); }

        // Store the search results and the currently selected search item.
        let matches: Rc<RefCell<BTreeMap<ModelIndexWrapped, Option<ModelIndexWrapped>>>> = Rc::new(RefCell::new(BTreeMap::new()));
        let position: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));

        // The data here represents "pattern", "flags to search", "column (-1 for all)".
        let search_data: Rc<RefCell<(String, Flags<MatchFlag>, i32)>> = Rc::new(RefCell::new(("".to_owned(), Flags::from_enum(MatchFlag::Contains), -1)));

        // Action to update the search stuff when needed.
        let update_search_stuff = Action::new(()).into_raw();

        // Build the columns. If we have a model from before, use it to paint our cells as they were last time we painted them.
        Self::build_columns(table_view, table_view_frozen, model, &table_definition, enable_header_popups.clone());

        {
            let mut table_state_data = table_state_data.borrow_mut();
            let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
            let undo_model = table_state_data.undo_model;

            if unsafe { undo_model.as_ref().unwrap().row_count(()) > 0 && undo_model.as_ref().unwrap().column_count(()) > 0 } { load_colors_from_undo_model(model, undo_model); }
            else { update_undo_model(model, undo_model); }
        }

        // If we want to let the columns resize themselfs...
        if SETTINGS.lock().unwrap().settings_bool["adjust_columns_to_content"] {
            unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
        }

        // Action to make the delete button delete contents.
        let smart_delete = Action::new(()).into_raw();

        // Create the Contextual Menu for the TableView.
        let mut context_menu = Menu::new(());
        let context_menu_add = context_menu.add_action(&QString::from_std_str("&Add Row"));
        let context_menu_insert = context_menu.add_action(&QString::from_std_str("&Insert Row"));
        let context_menu_delete = context_menu.add_action(&QString::from_std_str("&Delete Row"));

        let mut context_menu_apply_submenu = Menu::new(&QString::from_std_str("A&pply..."));
        let context_menu_apply_maths_to_selection = context_menu_apply_submenu.add_action(&QString::from_std_str("&Apply Maths to Selection"));
        let context_menu_rewrite_selection = context_menu_apply_submenu.add_action(&QString::from_std_str("&Rewrite Selection"));

        let mut context_menu_clone_submenu = Menu::new(&QString::from_std_str("&Clone..."));
        let context_menu_clone = context_menu_clone_submenu.add_action(&QString::from_std_str("&Clone and Insert"));
        let context_menu_clone_and_append = context_menu_clone_submenu.add_action(&QString::from_std_str("Clone and &Append"));
        
        let mut context_menu_copy_submenu = Menu::new(&QString::from_std_str("&Copy..."));
        let context_menu_copy = context_menu_copy_submenu.add_action(&QString::from_std_str("&Copy"));
        let context_menu_copy_as_lua_table = context_menu_copy_submenu.add_action(&QString::from_std_str("&Copy as &LUA Table"));

        let mut context_menu_paste_submenu = Menu::new(&QString::from_std_str("&Paste..."));
        let context_menu_paste = context_menu_paste_submenu.add_action(&QString::from_std_str("&Paste"));
        let context_menu_paste_as_new_lines = context_menu_paste_submenu.add_action(&QString::from_std_str("&Paste as New Rows"));
        let context_menu_paste_to_fill_selection = context_menu_paste_submenu.add_action(&QString::from_std_str("&Paste to Fill Selection"));

        let context_menu_search = context_menu.add_action(&QString::from_std_str("&Search"));
        let context_menu_sidebar = context_menu.add_action(&QString::from_std_str("Si&debar"));

        let context_menu_import = context_menu.add_action(&QString::from_std_str("&Import"));
        let context_menu_export = context_menu.add_action(&QString::from_std_str("&Export"));

        let context_menu_selection_invert = context_menu.add_action(&QString::from_std_str("Inver&t Selection"));
        
        let context_menu_undo = context_menu.add_action(&QString::from_std_str("&Undo"));
        let context_menu_redo = context_menu.add_action(&QString::from_std_str("&Redo"));

        // Set the shortcuts for these actions.
        unsafe { context_menu_add.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["add_row"]))); }
        unsafe { context_menu_insert.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["insert_row"]))); }
        unsafe { context_menu_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["delete_row"]))); }
        unsafe { context_menu_apply_maths_to_selection.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["apply_maths_to_selection"]))); }
        unsafe { context_menu_rewrite_selection.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["rewrite_selection"]))); }
        unsafe { context_menu_clone.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["clone_row"]))); }
        unsafe { context_menu_clone_and_append.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["clone_and_append_row"]))); }
        unsafe { context_menu_copy.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["copy"]))); }
        unsafe { context_menu_copy_as_lua_table.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["copy_as_lua_table"]))); }
        unsafe { context_menu_paste.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["paste"]))); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["paste_as_new_row"]))); }
        unsafe { context_menu_paste_to_fill_selection.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["paste_to_fill_selection"]))); }
        unsafe { context_menu_selection_invert.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["selection_invert"]))); }
        unsafe { context_menu_search.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["search"]))); }
        unsafe { context_menu_sidebar.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["sidebar"]))); }
        unsafe { context_menu_import.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["import_tsv"]))); }
        unsafe { context_menu_export.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["export_tsv"]))); }
        unsafe { smart_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["smart_delete"]))); }
        unsafe { context_menu_undo.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["undo"]))); }
        unsafe { context_menu_redo.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_table["redo"]))); }

        // Set the shortcuts to only trigger in the Table.
        unsafe { context_menu_add.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_insert.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_apply_maths_to_selection.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_rewrite_selection.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_clone.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_clone_and_append.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_copy.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_copy_as_lua_table.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_paste.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_paste_to_fill_selection.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_selection_invert.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_search.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_sidebar.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_import.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_export.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { smart_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_undo.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_redo.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }

        // Add the actions to the TableView, so the shortcuts work.
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_add); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_insert); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_delete); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_apply_maths_to_selection); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_rewrite_selection); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_clone); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_clone_and_append); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_copy); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_copy_as_lua_table); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_paste); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_paste_as_new_lines); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_paste_to_fill_selection); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_selection_invert); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_search); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_sidebar); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_import); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_export); }
        unsafe { table_view.as_mut().unwrap().add_action(smart_delete); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_undo); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_redo); }

        // Status Tips for the actions.
        unsafe { context_menu_add.as_mut().unwrap().set_status_tip(&QString::from_std_str("Add an empty row at the end of the table.")); }
        unsafe { context_menu_insert.as_mut().unwrap().set_status_tip(&QString::from_std_str("Insert an empty row just above the one selected.")); }
        unsafe { context_menu_delete.as_mut().unwrap().set_status_tip(&QString::from_std_str("Delete all the selected rows.")); }
        unsafe { context_menu_apply_maths_to_selection.as_mut().unwrap().set_status_tip(&QString::from_std_str("Apply a simple mathematical operation to every cell in the selected cells.")); }
        unsafe { context_menu_rewrite_selection.as_mut().unwrap().set_status_tip(&QString::from_std_str("Rewrite the selected cells using a pattern.")); }
        unsafe { context_menu_clone.as_mut().unwrap().set_status_tip(&QString::from_std_str("Duplicate the selected rows and insert the new rows under the original ones.")); }
        unsafe { context_menu_clone_and_append.as_mut().unwrap().set_status_tip(&QString::from_std_str("Duplicate the selected rows and append the new rows at the end of the table.")); }
        unsafe { context_menu_copy.as_mut().unwrap().set_status_tip(&QString::from_std_str("Copy whatever is selected to the Clipboard.")); }
        unsafe { context_menu_copy_as_lua_table.as_mut().unwrap().set_status_tip(&QString::from_std_str("Turns the entire DB Table into a LUA Table and copies it to the clipboard.")); }
        unsafe { context_menu_paste.as_mut().unwrap().set_status_tip(&QString::from_std_str("Try to paste whatever is in the Clipboard. If the data of a cell is incompatible with the content to paste, the cell is ignored.")); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().set_status_tip(&QString::from_std_str("Try to paste whatever is in the Clipboard as new lines at the end of the table. Does nothing if the data is not compatible with the cell.")); }
        unsafe { context_menu_paste_to_fill_selection.as_mut().unwrap().set_status_tip(&QString::from_std_str("Try to paste whatever is in the Clipboard in EVERY CELL selected. Does nothing if the data is not compatible with the cell.")); }
        unsafe { context_menu_selection_invert.as_mut().unwrap().set_status_tip(&QString::from_std_str("Inverts the current selection.")); }
        unsafe { context_menu_search.as_mut().unwrap().set_status_tip(&QString::from_std_str("Search what you want in the table. Also allows you to replace coincidences.")); }
        unsafe { context_menu_sidebar.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open/Close the sidebar with the controls to hide/show/freeze columns.")); }
        unsafe { context_menu_import.as_mut().unwrap().set_status_tip(&QString::from_std_str("Import a TSV file into this table, replacing all the data.")); }
        unsafe { context_menu_export.as_mut().unwrap().set_status_tip(&QString::from_std_str("Export this table's data into a TSV file.")); }
        unsafe { context_menu_undo.as_mut().unwrap().set_status_tip(&QString::from_std_str("A classic.")); }
        unsafe { context_menu_redo.as_mut().unwrap().set_status_tip(&QString::from_std_str("Another classic.")); }

        // Insert some separators to space the menu, and the paste submenu.
        unsafe { context_menu.insert_separator(context_menu_search); }
        unsafe { context_menu.insert_menu(context_menu_search, context_menu_apply_submenu.into_raw()); }
        unsafe { context_menu.insert_menu(context_menu_search, context_menu_clone_submenu.into_raw()); }
        unsafe { context_menu.insert_menu(context_menu_search, context_menu_copy_submenu.into_raw()); }
        unsafe { context_menu.insert_menu(context_menu_search, context_menu_paste_submenu.into_raw()); }
        unsafe { context_menu.insert_separator(context_menu_search); }
        unsafe { context_menu.insert_separator(context_menu_import); }
        unsafe { context_menu.insert_separator(context_menu_sidebar); }
        unsafe { context_menu.insert_separator(context_menu_undo); }

        // Create the "Hide/Show" and "Freeze/Unfreeze" slots and actions and connect them.
        let mut slots_hide_show_column = vec![];
        let mut slots_freeze_unfreeze_column = vec![];
        let actions_hide_show_column: Rc<RefCell<Vec<*mut CheckBox>>> = Rc::new(RefCell::new(vec![]));
        let actions_freeze_unfreeze_column: Rc<RefCell<Vec<*mut CheckBox>>> = Rc::new(RefCell::new(vec![]));

        let header_column = Label::new(&QString::from_std_str("<b><i>Column Name</i></b>")).into_raw();
        let header_hidden = Label::new(&QString::from_std_str("<b><i>Hidden</i></b>")).into_raw();
        let header_frozen = Label::new(&QString::from_std_str("<b><i>Frozen</i></b>")).into_raw();

        unsafe { sidebar_grid.as_mut().unwrap().add_widget((header_column as *mut Widget, 0, 0, 1, 1)); }
        unsafe { sidebar_grid.as_mut().unwrap().add_widget((header_hidden as *mut Widget, 0, 1, 1, 1)); }
        unsafe { sidebar_grid.as_mut().unwrap().add_widget((header_frozen as *mut Widget, 0, 2, 1, 1)); }

        unsafe { sidebar_grid.as_mut().unwrap().set_alignment((header_column as *mut Widget, Flags::from_enum(AlignmentFlag::HCenter))); } 
        unsafe { sidebar_grid.as_mut().unwrap().set_alignment((header_hidden as *mut Widget, Flags::from_enum(AlignmentFlag::HCenter))); } 
        unsafe { sidebar_grid.as_mut().unwrap().set_alignment((header_frozen as *mut Widget, Flags::from_enum(AlignmentFlag::HCenter))); } 
        for (index, column) in table_definition.fields.iter().enumerate() {

            // Hide all columns in the frozen table by default.
            unsafe { table_view_frozen.as_mut().unwrap().set_column_hidden(index as i32, true); }

            // Prepare the hide/show slot, taking into account the Frozen TableView when hiding/showing.
            // Logic here: If we hide something, it cannot be frozen.
            let hide_show_slot = SlotCInt::new(clone!(
                packed_file_path,
                actions_freeze_unfreeze_column => move |state| {


                    // Hide the column and disable Freezing for it.
                    let state = if state == 2 { true } else { false };
                    unsafe { table_view.as_mut().unwrap().set_column_hidden(index as i32, state); }
                    unsafe { actions_freeze_unfreeze_column.borrow()[index].as_mut().unwrap().set_enabled(!state); }

                    // Update the state of the column in the table history.
                    if let Some(state_ui) = TABLE_STATES_UI.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
                        state_ui.columns_state.visual_history.push(VisualHistory::ColumnHidden(state, index as i32));
                    }
                }
            ));

            // Logic here: If we freeze something, it cannot be hidden.
            let freeze_unfreeze_slot = SlotCInt::new(clone!(
                packed_file_path,
                actions_hide_show_column => move |state| {
                    let state = if state == 2 { true } else { false };
                    unsafe { table_view_frozen.as_mut().unwrap().set_column_hidden(index as i32, !state); }
                    unsafe { actions_hide_show_column.borrow()[index].as_mut().unwrap().set_enabled(!state); }

                    // Due to locking issues, we have to block the header, then move the columns.
                    let header = unsafe { table_view.as_ref().unwrap().horizontal_header() };
                    let mut blocker = unsafe { SignalBlocker::new( header.as_mut().unwrap().static_cast_mut() as &mut Object) };
                    if let Some(state_ui) = TABLE_STATES_UI.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
                        let visual_index_source = unsafe { header.as_ref().unwrap().visual_index(index as i32) };

                        // If we're freezing, just move the columns on both tables, and then keep track of the movement.
                        if state {
                            unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().move_section(visual_index_source, 0); }
                            unsafe { table_view_frozen.as_mut().unwrap().horizontal_header().as_mut().unwrap().move_section(visual_index_source, 0); }
                            state_ui.columns_state.visual_history.push(VisualHistory::ColumnFrozen(true, index as i32, visual_index_source));
                        }

                        // Otherwise, get the last time we moved that column, then move it to the position where it was before.
                        else {
                            let mut visual_index_destination = 0;
                            for i in state_ui.columns_state.visual_history.iter().rev() {
                                match i {
                                    VisualHistory::ColumnFrozen(_, logical_index, original_position) => {
                                        if *logical_index == index as i32 {
                                            visual_index_destination = *original_position;
                                            break;
                                        }
                                    },
                                    _ => {},
                                }
                            }
                            unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().move_section(visual_index_source, visual_index_destination); }
                            unsafe { table_view_frozen.as_mut().unwrap().horizontal_header().as_mut().unwrap().move_section(visual_index_source, visual_index_destination); }
                            state_ui.columns_state.visual_history.push(VisualHistory::ColumnFrozen(false, index as i32,  visual_index_destination));
                        }
                    }
                    blocker.unblock();

                    // Stupid problems require stupid solutions. This fixes the "Frozen Table doesn't update until I resize it" bug.
                    let column_width = unsafe { table_view.as_mut().unwrap().column_width(index as i32) };
                    unsafe { header.as_mut().unwrap().resize_section(index as i32, column_width + 1); }
                    unsafe { header.as_mut().unwrap().resize_section(index as i32, column_width - 1); }
                }
            ));

            let column_name = Label::new(&QString::from_std_str(&Self::clean_column_names(&column.field_name)));
            let hide_show_checkbox = CheckBox::new(()).into_raw();
            let freeze_unfreeze_checkbox = CheckBox::new(()).into_raw();

            unsafe { hide_show_checkbox.as_mut().unwrap().signals().state_changed().connect(&hide_show_slot); }
            unsafe { freeze_unfreeze_checkbox.as_mut().unwrap().signals().state_changed().connect(&freeze_unfreeze_slot); }
            unsafe { sidebar_grid.as_mut().unwrap().add_widget((column_name.into_raw() as *mut Widget, (index + 1) as i32, 0, 1, 1)); }
            unsafe { sidebar_grid.as_mut().unwrap().add_widget((hide_show_checkbox as *mut Widget, (index + 1) as i32, 1, 1, 1)); }
            unsafe { sidebar_grid.as_mut().unwrap().add_widget((freeze_unfreeze_checkbox as *mut Widget, (index + 1) as i32, 2, 1, 1)); } 
            
            unsafe { sidebar_grid.as_mut().unwrap().set_alignment((hide_show_checkbox as *mut Widget, Flags::from_enum(AlignmentFlag::HCenter))); } 
            unsafe { sidebar_grid.as_mut().unwrap().set_alignment((freeze_unfreeze_checkbox as *mut Widget, Flags::from_enum(AlignmentFlag::HCenter))); } 

            slots_hide_show_column.push(hide_show_slot);
            slots_freeze_unfreeze_column.push(freeze_unfreeze_slot);
            actions_hide_show_column.borrow_mut().push(hide_show_checkbox);
            actions_freeze_unfreeze_column.borrow_mut().push(freeze_unfreeze_checkbox);
        }

        // Slots for the TableView...
        let slots = Self {
            slot_column_moved: SlotCIntCIntCInt::new(clone!(
                packed_file_path => move |_, visual_old, visual_new| {
                    if let Some(state) = TABLE_STATES_UI.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
                        state.columns_state.visual_history.push(VisualHistory::ColumnMoved(visual_old, visual_new));
                        unsafe { table_view_frozen.as_mut().unwrap().horizontal_header().as_mut().unwrap().move_section(visual_old, visual_new); }
                    }
                }
            )),

            slot_sort_order_column_changed: SlotCIntQtCoreQtSortOrder::new(clone!(
                packed_file_path => move |column, _| {
                    if let Ok(mut state) = TABLE_STATES_UI.try_lock() {
                        if let Some(state) = state.get_mut(&*packed_file_path.borrow()) {
                            let mut needs_cleaning = false;
                            
                            // We only change the order if it's less than 2. Otherwise, we reset it.
                            let mut old_order = if state.columns_state.sorting_column.0 == column { 
                                state.columns_state.sorting_column.1 
                            } else { 0 };

                            if old_order < 2 {
                                old_order += 1;

                                if old_order == 0 { state.columns_state.sorting_column = (-1, old_order); }
                                else { state.columns_state.sorting_column = (column, old_order); }
                            }
                            else {
                                needs_cleaning = true;
                                old_order = -1;
                                state.columns_state.sorting_column = (-1, old_order);   
                            }

                            if needs_cleaning {
                                unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_sort_indicator(-1, SortOrder::Ascending) };
                                unsafe { table_view_frozen.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_sort_indicator(-1, SortOrder::Ascending) };
                            }
                        }
                    }
                }
            )),

            slot_undo: SlotNoArgs::new(clone!(
                global_search_explicit_paths,
                dependency_data,
                packed_file_path,
                app_ui,
                sender_qt,
                sender_qt_data,
                receiver_qt,
                enable_header_popups,
                undo_lock,
                save_lock,
                table_type,
                table_definition,
                table_state_data => move || {
                    {
                        let mut table_state_data = table_state_data.borrow_mut();
                        let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                        Self::undo_redo(
                            &app_ui,
                            &dependency_data,
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &packed_file_path,
                            table_view,
                            table_view_frozen,
                            model,
                            filter_model,
                            &mut table_state_data.undo_history,
                            &mut table_state_data.redo_history,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
                            &undo_lock,
                            &save_lock,
                            &table_definition,
                            &table_type,
                            enable_header_popups.clone()
                        );

                        update_undo_model(model, table_state_data.undo_model);
                    }
                    unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                    unsafe { update_search_stuff.as_mut().unwrap().trigger(); }
                }
            )),

            slot_redo: SlotNoArgs::new(clone!(
                global_search_explicit_paths,
                dependency_data,
                packed_file_path,
                app_ui,
                sender_qt,
                sender_qt_data,
                receiver_qt,
                enable_header_popups,
                undo_lock,
                save_lock,
                table_type,
                table_definition,
                table_state_data => move || {
                    {
                        let mut table_state_data = table_state_data.borrow_mut();
                        let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                        Self::undo_redo(
                            &app_ui,
                            &dependency_data,
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &packed_file_path,
                            table_view,
                            table_view_frozen,
                            model,
                            filter_model,
                            &mut table_state_data.redo_history,
                            &mut table_state_data.undo_history,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
                            &undo_lock,
                            &save_lock,
                            &table_definition,
                            &table_type,
                            enable_header_popups.clone()
                        );

                        update_undo_model(model, table_state_data.undo_model); 
                    }
                    unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                    unsafe { update_search_stuff.as_mut().unwrap().trigger(); }
                }
            )),

            slot_undo_redo_enabler: SlotNoArgs::new(clone!(
                sender_qt,
                sender_qt_data,
                receiver_qt,
                app_ui,
                table_type,
                table_state_data,
                packed_file_path => move || { 
                    let table_state_data = table_state_data.borrow_mut();
                    let table_state_data = table_state_data.get(&*packed_file_path.borrow()).unwrap();
                    unsafe {
                        if table_state_data.undo_history.is_empty() { 
                            context_menu_undo.as_mut().unwrap().set_enabled(false);
                            let tree_path_type = match *table_type.borrow() {
                                TableType::DependencyManager(_) => TreePathType::PackFile,
                                TableType::DB(_) | TableType::LOC(_) => TreePathType::File(packed_file_path.borrow().to_vec()), 
                            };
                            update_treeview(
                                &sender_qt,
                                &sender_qt_data,
                                &receiver_qt,
                                &app_ui,
                                app_ui.folder_tree_view,
                                Some(app_ui.folder_tree_filter),
                                app_ui.folder_tree_model,
                                TreeViewOperation::Undo(vec![tree_path_type]),
                            );
                        }
                        else { context_menu_undo.as_mut().unwrap().set_enabled(true); }
                        
                        if table_state_data.redo_history.is_empty() { context_menu_redo.as_mut().unwrap().set_enabled(false); }
                        else { context_menu_redo.as_mut().unwrap().set_enabled(true); }
                    }
                }
            )),

            slot_context_menu: SlotQtCorePointRef::new(move |_| { context_menu.exec2(&Cursor::pos()); }),
            slot_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(clone!(
                table_definition => move |_,_| {

                    // Turns out that this slot doesn't give the the amount of selected items, so we have to get them ourselfs.
                    let indexes = unsafe { filter_model.as_mut().unwrap().map_selection_to_source(&table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };

                    // If we have something selected, enable these actions.
                    if indexes.count(()) > 0 {
                        unsafe {
                            context_menu_clone.as_mut().unwrap().set_enabled(true);
                            context_menu_clone_and_append.as_mut().unwrap().set_enabled(true);
                            context_menu_copy.as_mut().unwrap().set_enabled(true);
                            context_menu_delete.as_mut().unwrap().set_enabled(true);
                            context_menu_rewrite_selection.as_mut().unwrap().set_enabled(true);
                        
                            // The "Apply" actions have to be enabled only when all the indexes are valid for the operation. 
                            let mut columns = vec![];
                            for index in 0..indexes.count(()) {
                                let model_index = indexes.at(index);
                                if model_index.is_valid() { columns.push(model_index.column()); }
                            }

                            columns.sort();
                            columns.dedup();
                            
                            let mut can_apply = true;
                            for column in &columns {
                                let field_type = &table_definition.fields[*column as usize].field_type;
                                if *field_type != FieldType::Boolean { continue }
                                else { can_apply = false; break } 
                            }
                            context_menu_apply_maths_to_selection.as_mut().unwrap().set_enabled(can_apply);
                        }
                    }

                    // Otherwise, disable them.
                    else {
                        unsafe {
                            context_menu_apply_maths_to_selection.as_mut().unwrap().set_enabled(false);
                            context_menu_rewrite_selection.as_mut().unwrap().set_enabled(false);
                            context_menu_clone.as_mut().unwrap().set_enabled(false);
                            context_menu_clone_and_append.as_mut().unwrap().set_enabled(false);
                            context_menu_copy.as_mut().unwrap().set_enabled(false);
                            context_menu_delete.as_mut().unwrap().set_enabled(false);
                        }
                    }
                }
            )),
            save_changes: SlotModelIndexRefModelIndexRefVectorVectorCIntRef::new(clone!(
                global_search_explicit_paths,
                packed_file_path,
                app_ui,
                table_definition,
                table_type,
                save_lock,
                receiver_qt,
                sender_qt,
                sender_qt_data => move |_,_,roles| {

                    // To avoid doing this multiple times due to the cell painting stuff, we need to check the role.
                    // This has to be allowed ONLY if the role is 0 (DisplayText), 2 (EditorText) or 10 (CheckStateRole).
                    // 16 is a role we use as an special trigger for this.
                    if roles.contains(&0) || roles.contains(&2) || roles.contains(&10) || roles.contains(&16) {

                        // For pasting, only save the last iteration of the paste.                        
                        if !*save_lock.borrow() {

                            // Thanks to validation, this should NEVER fail. If this fails, revise the validation stuff.
                            Self::save_to_packed_file(
                                &sender_qt,
                                &sender_qt_data,
                                &receiver_qt,
                                &app_ui,
                                &packed_file_path,
                                model,
                                &global_search_explicit_paths,
                                update_global_search_stuff,
                                &table_definition,
                                &mut table_type.borrow_mut(),
                            );

                            // Otherwise, update the needed stuff.
                            unsafe { update_search_stuff.as_mut().unwrap().trigger(); }
                        }
                    }
                }
            )),

            slot_item_changed: SlotStandardItemMutPtr::new(clone!(
                undo_lock,
                packed_file_path,
                table_type,
                save_lock,
                table_state_data,
                dependency_data,
                table_definition => move |item| {

                    // If we are NOT UNDOING, paint the item as edited and add the edition to the undo list.
                    if !*undo_lock.borrow() {
                        {
                            let mut table_state_data = table_state_data.borrow_mut();
                            let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                            let item_old = unsafe { &*table_state_data.undo_model.as_mut().unwrap().item((item.as_mut().unwrap().row(), item.as_mut().unwrap().column())) };
                            unsafe { table_state_data.undo_history.push(TableOperations::Editing(vec![((item.as_mut().unwrap().row(), item.as_mut().unwrap().column()), item_old.clone()); 1])); }
                            table_state_data.redo_history.clear();

                            // We block the saving for painting, so this doesn't get rettriggered again.
                            let mut blocker = unsafe { SignalBlocker::new(model.as_mut().unwrap().static_cast_mut() as &mut Object) };
                            unsafe { item.as_mut().unwrap().set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkYellow } else { GlobalColor::Yellow })); }
                            blocker.unblock();

                            // For pasting, only update the undo_model the last iteration of the paste.                        
                            if !*save_lock.borrow() {
                                update_undo_model(model, table_state_data.undo_model);
                            }
                        }

                        unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                    }

                    // If we have the dependency stuff enabled, check if it's a valid reference.
                    if SETTINGS.lock().unwrap().settings_bool["use_dependency_checker"] {
                        let column = unsafe { item.as_mut().unwrap().column() };
                        if table_definition.fields[column as usize].field_is_reference.is_some() {
                            Self::check_references(&dependency_data, column, item);
                        }
                    }

                    // If we are editing the Dependency Manager, check for PackFile errors too.
                    if let TableType::DependencyManager(_) = *table_type.borrow() { Self::check_dependency_packfile_errors(model); }
                }
            )),

            slot_row_filter_change_text: SlotStringRef::new(clone!(
                packed_file_path => move |filter_text| {
                    Self::filter_table(
                        Some(QString::from_std_str(filter_text.to_std_string())),
                        None,
                        None,
                        filter_model,
                        row_filter_line_edit,
                        row_filter_column_selector,
                        row_filter_case_sensitive_button,
                        update_search_stuff,
                        &packed_file_path,
                    ); 
                }
            )),
            slot_row_filter_change_column: SlotCInt::new(clone!(
                packed_file_path => move |index| {
                    Self::filter_table(
                        None,
                        Some(index),
                        None,
                        filter_model,
                        row_filter_line_edit,
                        row_filter_column_selector,
                        row_filter_case_sensitive_button,
                        update_search_stuff,
                        &packed_file_path,
                    ); 
                }
            )),
            slot_row_filter_change_case_sensitive: SlotBool::new(clone!(
                packed_file_path => move |case_sensitive| {
                    Self::filter_table(
                        None,
                        None,
                        Some(case_sensitive),
                        filter_model,
                        row_filter_line_edit,
                        row_filter_column_selector,
                        row_filter_case_sensitive_button,
                        update_search_stuff,
                        &packed_file_path,
                    ); 
                }
            )),

            slot_context_menu_add: SlotBool::new(clone!(
                global_search_explicit_paths,
                packed_file_path,
                app_ui,
                table_state_data,
                table_type,
                receiver_qt,
                sender_qt,
                sender_qt_data,
                table_definition => move |_| {

                    // Create the row and append it.
                    let rows = create_empty_rows(&table_definition, 1);
                    for row in &rows { unsafe { model.as_mut().unwrap().append_row(row); } }

                    // Save, so there are no discrepances between the normal and undo models.
                    Self::save_to_packed_file(
                        &sender_qt,
                        &sender_qt_data,
                        &receiver_qt,
                        &app_ui,
                        &packed_file_path,
                        model,
                        &global_search_explicit_paths,
                        update_global_search_stuff,
                        &table_definition,
                        &mut table_type.borrow_mut(),
                    );

                    // Update the search stuff, if needed.
                    unsafe { update_search_stuff.as_mut().unwrap().trigger(); }

                    // Add the operation to the undo history.
                    {
                        let mut table_state_data = table_state_data.borrow_mut();
                        let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                        unsafe { table_state_data.undo_history.push(TableOperations::AddRows(vec![model.as_mut().unwrap().row_count(()) - 1; 1])); }
                        table_state_data.redo_history.clear();
                        update_undo_model(model, table_state_data.undo_model); 
                    }
                    unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                }
            )),
            slot_context_menu_insert: SlotBool::new(clone!(
                global_search_explicit_paths,
                packed_file_path,
                app_ui,
                table_state_data,
                table_type,
                receiver_qt,
                sender_qt,
                sender_qt_data,
                table_definition => move |_| {

                    // Get the indexes ready for battle.
                    let indexes = unsafe { table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection().indexes() };
                    let indexes_sorted = (0..indexes.count(())).map(|x| indexes.at(x)).collect::<Vec<&ModelIndex>>();
                    let mut indexes_sorted = get_real_indexes(&indexes_sorted, filter_model);
                    sort_indexes_by_model(&mut indexes_sorted);
                    dedup_indexes_per_row(&mut indexes_sorted);
                    let mut row_numbers = vec![];

                    // If nothing is selected, we just append the row at the end.
                    if indexes_sorted.is_empty() {
                        let rows = create_empty_rows(&table_definition, 1);
                        unsafe { model.as_mut().unwrap().append_row(&rows[0]); } 
                        row_numbers.push(unsafe { model.as_mut().unwrap().row_count(()) - 1 });
                    }
                
                    for index in indexes_sorted.iter().rev() {
                        row_numbers.push(index.row());
                        let rows = create_empty_rows(&table_definition, 1);
                        unsafe { model.as_mut().unwrap().insert_row((index.row(), &rows[0])); }
                    }

                    // Save, so there are no discrepances between the normal and undo models.
                    Self::save_to_packed_file(
                        &sender_qt,
                        &sender_qt_data,
                        &receiver_qt,
                        &app_ui,
                        &packed_file_path,
                        model,
                        &global_search_explicit_paths,
                        update_global_search_stuff,
                        &table_definition,
                        &mut table_type.borrow_mut(),
                    );

                    // Update the search stuff, if needed.
                    unsafe { update_search_stuff.as_mut().unwrap().trigger(); }
                    {

                        // The undo mode needs this reversed.
                        row_numbers.reverse();
                        let mut table_state_data = table_state_data.borrow_mut();
                        let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                        table_state_data.undo_history.push(TableOperations::AddRows(row_numbers));
                        table_state_data.redo_history.clear();
                        update_undo_model(model, table_state_data.undo_model); 
                    }
                    unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                }
            )),
            slot_context_menu_delete: SlotBool::new(clone!(
                global_search_explicit_paths,
                packed_file_path,
                app_ui,
                table_state_data,
                table_definition,
                table_type,
                receiver_qt,
                sender_qt,
                sender_qt_data => move |_| {

                    // Get all the selected rows.
                    let indexes = unsafe { filter_model.as_mut().unwrap().map_selection_to_source(&table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };
                    let mut rows: Vec<i32> = vec![];
                    for index in 0..indexes.size() {
                        let model_index = indexes.at(index);
                        if model_index.is_valid() { rows.push(model_index.row()); }
                    }

                    // Dedup the list and reverse it.
                    rows.sort();
                    rows.dedup();
                    rows.reverse();

                    // Split the row list in consecutive rows, get their data, and remove them in batches.
                    let mut rows_splitted = vec![];
                    let mut current_row_pack = vec![];
                    let mut current_row_index = -2;
                    for (index, row) in rows.iter().enumerate() {

                        let mut items = vec![];                        
                        for column in 0..unsafe { model.as_mut().unwrap().column_count(()) } {
                            let item = unsafe { &*model.as_mut().unwrap().item((*row, column)) };
                            items.push(item.clone());
                        }

                        if (*row == current_row_index - 1) || index == 0 { 
                            current_row_pack.push((*row, items)); 
                            current_row_index = *row;
                        }
                        else {
                            current_row_pack.reverse();
                            rows_splitted.push(current_row_pack.to_vec());
                            current_row_pack.clear();
                            current_row_pack.push((*row, items)); 
                            current_row_index = *row;
                        }
                    }
                    current_row_pack.reverse();
                    rows_splitted.push(current_row_pack);

                    for row_pack in rows_splitted.iter() {
                        unsafe { model.as_mut().unwrap().remove_rows((row_pack[0].0, row_pack.len() as i32)); }
                    }
                    
                    // If we deleted something, try to save the PackedFile to the main PackFile.
                    if !rows.is_empty() {
                        Self::save_to_packed_file(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &app_ui,
                            &packed_file_path,
                            model,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
                            &table_definition,
                            &mut table_type.borrow_mut(),
                        );

                        // Update the search stuff, if needed.
                        unsafe { update_search_stuff.as_mut().unwrap().trigger(); }

                        {
                            let mut table_state_data = table_state_data.borrow_mut();
                            let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                            rows_splitted.reverse();
                            table_state_data.undo_history.push(TableOperations::RemoveRows(rows_splitted));
                            table_state_data.redo_history.clear();
                            update_undo_model(model, table_state_data.undo_model); 
                        }

                        unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                    }
                }
            )),

            slot_context_menu_apply_maths_to_selection: SlotBool::new(clone!(
                packed_file_path,
                table_state_data,
                table_definition,
                app_ui => move |_| {

                    // If we got an operation, get all the cells in the selection, try to apply the operation to them and,
                    // if the resulting value is valid in each of them, apply it.
                    if let Some(operation) = create_apply_maths_dialog(&app_ui) {

                        let mut results = 0;
                        let indexes_visual = unsafe { table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection().indexes() };
                        let indexes_visual = (0..indexes_visual.count(())).map(|x| indexes_visual.at(x)).collect::<Vec<&ModelIndex>>();
                        let indexes_real = get_real_indexes(&indexes_visual, filter_model);
                        for index in indexes_real {
                            if index.is_valid() { 

                                // First, we replace {x} with our current value. Then, we try to parse with meval.
                                // And finally, we try to put the new value in the cell.
                                let current_value = unsafe { model.as_ref().unwrap().item_from_index(&index).as_ref().unwrap().data(2).to_string().to_std_string() };
                                let real_operation = operation.replace("{x}", &current_value);

                                // We only do this if the current value is a valid number.
                                if let Ok(result) = meval::eval_str(&real_operation) {
                                    let mut is_valid = false;
                                    
                                    // If we got a current value and it's different, it's a valid cell.
                                    if let Ok(current_value) = current_value.parse::<f64>() {
                                        if (result - current_value).abs() >= std::f64::EPSILON { 
                                            is_valid = true;
                                        }
                                    }

                                    // Otherwise, it's a change over a string. Allow it.
                                    else { is_valid = true; }
                                    if is_valid {    
                                        match table_definition.fields[index.column() as usize].field_type {
                                            FieldType::Float => unsafe { model.as_mut().unwrap().item_from_index(&index).as_mut().unwrap().set_data((&Variant::new2(result as f32), 2)) }
                                            FieldType::Integer => unsafe { model.as_mut().unwrap().item_from_index(&index).as_mut().unwrap().set_data((&Variant::new0(result as i32), 2)) },
                                            FieldType::LongInteger => unsafe { model.as_mut().unwrap().item_from_index(&index).as_mut().unwrap().set_data((&Variant::new2(result as i64), 2)) },
                                            
                                            FieldType::StringU8 |
                                            FieldType::StringU16 |
                                            FieldType::OptionalStringU8 |
                                            FieldType::OptionalStringU16 => unsafe { model.as_mut().unwrap().item_from_index(&index).as_mut().unwrap().set_text(&QString::from_std_str(&format!("{}", result))) },
                                            _ => continue,
                                        }
                                        results += 1;
                                    }
                                }
                            }
                        }

                        // If we finished doing maths, fix the undo history to have all the previous changes merged into one.
                        if results > 0 {
                            {
                                let mut table_state_data = table_state_data.borrow_mut();
                                let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();

                                // If we finished appling prefixes, fix the undo history to have all the previous changes merged into one.
                                // Keep in mind that `None` results should be ignored here.
                                let len = table_state_data.undo_history.len();
                                let mut edits_data = vec![];
                                
                                {
                                    let mut edits = table_state_data.undo_history.drain((len - results)..);
                                    for edit in &mut edits { if let TableOperations::Editing(mut edit) = edit { edits_data.append(&mut edit); }}
                                }

                                table_state_data.undo_history.push(TableOperations::Editing(edits_data));
                                table_state_data.redo_history.clear();
                                update_undo_model(model, table_state_data.undo_model); 
                            }

                            unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                        }
                    }
                }
            )),

            slot_context_menu_rewrite_selection: SlotBool::new(clone!(
                packed_file_path,
                table_state_data,
                table_definition,
                app_ui => move |_| {

                    // If we got a sequence, get all the cells in the selection, try to apply it to them.
                    if let Some(mut sequence) = create_rewrite_selection_dialog(&app_ui) {

                        // For some reason Qt adds & sometimes, so remove it if you found it.
                        if let Some(index) = sequence.find('&') { sequence.remove(index); }

                        // Get all the selected cells. We can rewrite any kind of cell (except Booleans),
                        // so we have to do a first pass to ensure everything is valid before applying the data.
                        let indexes = unsafe { filter_model.as_mut().unwrap().map_selection_to_source(&table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };
                        let mut results = vec![];
                        for index in 0..indexes.count(()) {
                            let model_index = indexes.at(index);

                            // Always check this is valid. Otherwise this can and will crash if the filter goes the wrong way.
                            if model_index.is_valid() { 
                                let item = unsafe { model.as_ref().unwrap().item_from_index(model_index).as_ref().unwrap() };
                                let column = item.column();
                                let column_type = table_definition.fields[column as usize].field_type;
                                let text = match column_type {

                                    // As I said, we skip booleans.
                                    FieldType::Boolean => continue,
                                    FieldType::Float |
                                    FieldType::Integer |
                                    FieldType::LongInteger |
                                    FieldType::StringU8 |
                                    FieldType::StringU16 |
                                    FieldType::OptionalStringU8 |
                                    FieldType::OptionalStringU16 => item.text().to_std_string(),
                                };

                                // If any of the new texts is incompatible with his cells, skip it.
                                let replaced_text = sequence.to_owned().replace("{x}", &text).replace("{X}", &text);
                                match column_type {
                                    FieldType::Boolean => continue,
                                    FieldType::Float => if replaced_text.parse::<f32>().is_err() { continue; }
                                    FieldType::Integer => if replaced_text.parse::<i32>().is_err() { continue; }
                                    FieldType::LongInteger => if replaced_text.parse::<i64>().is_err() { continue; }
                                    FieldType::StringU8 |
                                    FieldType::StringU16 |
                                    FieldType::OptionalStringU8 |
                                    FieldType::OptionalStringU16 => {},
                                };

                                results.push((model_index, replaced_text));
                            }
                        }

                        // Then iterate again over every result applying the new value to the cell. Save the amount of changes.
                        let mut changed_cells = 0;
                        for (model_index, result) in results {
                            let item = unsafe { model.as_ref().unwrap().item_from_index(model_index).as_mut().unwrap() };
                            let column = item.column();
                            let column_type = table_definition.fields[column as usize].field_type;
                            match column_type {

                                // If we hit this, something above this is broken.
                                FieldType::Boolean => continue,

                                FieldType::Float => {
                                    let current_value = item.text().to_std_string();
                                    if *current_value != result {
                                        item.set_data((&Variant::new2(result.parse::<f32>().unwrap()), 2));
                                        changed_cells += 1;
                                    }
                                },

                                FieldType::Integer => {
                                    let current_value = item.text().to_std_string();
                                    if *current_value != result {
                                        item.set_data((&Variant::new0(result.parse::<i32>().unwrap()), 2));
                                        changed_cells += 1;
                                    }
                                },

                                FieldType::LongInteger => {
                                    let current_value = item.text().to_std_string();
                                    if *current_value != result {
                                        item.set_data((&Variant::new2(result.parse::<i64>().unwrap()), 2));
                                        changed_cells += 1;
                                    }
                                },

                                FieldType::StringU8 |
                                FieldType::StringU16 |
                                FieldType::OptionalStringU8 |
                                FieldType::OptionalStringU16 => {
                                    let current_value = item.text().to_std_string();
                                    if *current_value != result {
                                        item.set_text(&QString::from_std_str(result));
                                        changed_cells += 1;
                                    }
                                }
                            }
                        }

                        {
                            let mut table_state_data = table_state_data.borrow_mut();
                            let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();

                            // If we finished rewriting cells, fix the undo history to have all the previous changes merged into one.
                            // Keep in mind that `None` results should be ignored here.
                            let len = table_state_data.undo_history.len();
                            let mut edits_data = vec![];
                            
                            {
                                let mut edits = table_state_data.undo_history.drain((len - changed_cells)..);
                                for edit in &mut edits { if let TableOperations::Editing(mut edit) = edit { edits_data.append(&mut edit); }}
                            }

                            table_state_data.undo_history.push(TableOperations::Editing(edits_data));
                            table_state_data.redo_history.clear();
                            update_undo_model(model, table_state_data.undo_model); 
                        }

                        unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                    }
                }
            )),

            slot_context_menu_clone: SlotBool::new(clone!(
                global_search_explicit_paths,
                packed_file_path,
                app_ui,
                table_definition,
                table_state_data,
                table_type,
                receiver_qt,
                sender_qt,
                sender_qt_data => move |_| {

                    // Get all the selected rows.
                    let indexes = unsafe { filter_model.as_mut().unwrap().map_selection_to_source(&table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };
                    let mut rows: Vec<i32> = vec![];
                    for index in 0..indexes.size() {
                        let model_index = indexes.at(index);
                        if model_index.is_valid() { rows.push(model_index.row()); }
                    }

                    // Dedup the list and reverse it.
                    rows.sort();
                    rows.dedup();
                    rows.reverse();

                    // For each row to clone, create a new one, duplicate the items and add the row under the old one.
                    for row in &rows {
                        let mut qlist = ListStandardItemMutPtr::new(());
                        for column in 0..table_definition.fields.len() {

                            // Get the original item and his clone.
                            let original_item = unsafe { model.as_mut().unwrap().item((*row, column as i32)) };
                            let item = unsafe { original_item.as_mut().unwrap().clone() };
                            unsafe { item.as_mut().unwrap().set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green })); }
                            unsafe { qlist.append_unsafe(&item); }
                        }

                        // Insert the new row after the original one.
                        unsafe { model.as_mut().unwrap().insert_row((row + 1, &qlist)); }
                    }

                    // If we cloned something, try to save the PackedFile to the main PackFile.
                    if !rows.is_empty() {
                        Self::save_to_packed_file(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &app_ui,
                            &packed_file_path,
                            model,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
                            &table_definition,
                            &mut table_type.borrow_mut(),
                        );

                        // Update the search stuff, if needed.
                        unsafe { update_search_stuff.as_mut().unwrap().trigger(); }

                        // Update the undo stuff. Cloned rows are their equivalent + 1 starting from the top, so we need to take that into account.
                        rows.iter_mut().rev().enumerate().for_each(|(y, x)| *x += 1 + y as i32);
                        {
                            let mut table_state_data = table_state_data.borrow_mut();
                            let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                            table_state_data.undo_history.push(TableOperations::AddRows(rows));
                            table_state_data.redo_history.clear();
                            update_undo_model(model, table_state_data.undo_model); 
                        }

                        unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                    }
                }
            )),

            slot_context_menu_clone_and_append: SlotBool::new(clone!(
                global_search_explicit_paths,
                packed_file_path,
                app_ui,
                table_definition,
                table_state_data,
                table_type,
                receiver_qt,
                sender_qt,
                sender_qt_data => move |_| {

                    // Get all the selected rows.
                    let indexes = unsafe { filter_model.as_mut().unwrap().map_selection_to_source(&table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };
                    let mut rows: Vec<i32> = vec![];
                    for index in 0..indexes.size() {
                        let model_index = indexes.at(index);
                        if model_index.is_valid() { rows.push(model_index.row()); }
                    }

                    // Dedup the list.
                    rows.sort();
                    rows.dedup();

                    // For each row to clone, create a new one, duplicate the items and add the row under the old one.
                    for row in &rows {
                        let mut qlist = ListStandardItemMutPtr::new(());
                        for column in 0..table_definition.fields.len() {

                            // Get the original item and his clone.
                            let original_item = unsafe { model.as_mut().unwrap().item((*row, column as i32)) };
                            let item = unsafe { original_item.as_mut().unwrap().clone() };
                            unsafe { item.as_mut().unwrap().set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green })); }
                            unsafe { qlist.append_unsafe(&item); }
                        }

                        // Insert the new row after the original one.
                        unsafe { model.as_mut().unwrap().append_row(&qlist); }
                    }

                    // If we cloned something, try to save the PackedFile to the main PackFile.
                    if !rows.is_empty() {
                        Self::save_to_packed_file(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &app_ui,
                            &packed_file_path,
                            model,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
                            &table_definition,
                            &mut table_type.borrow_mut(),
                        );

                        // Update the search stuff, if needed.
                        unsafe { update_search_stuff.as_mut().unwrap().trigger(); }

                        // Update the undo stuff. Cloned rows are the amount of rows - the amount of cloned rows.
                        let total_rows = unsafe { model.as_ref().unwrap().row_count(()) };
                        let range = (total_rows - rows.len() as i32..total_rows).rev().collect::<Vec<i32>>();
                        {
                            let mut table_state_data = table_state_data.borrow_mut();
                            let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                            table_state_data.undo_history.push(TableOperations::AddRows(range));
                            table_state_data.redo_history.clear();
                            update_undo_model(model, table_state_data.undo_model); 
                        }

                        unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                    }
                }
            )),

            slot_context_menu_copy: SlotBool::new(move |_| {

                // Get the current selection. As we need his visual order, we get it directly from the table/filter, NOT FROM THE MODEL.
                let indexes = unsafe { table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection().indexes() };
                let mut indexes_sorted = (0..indexes.count(())).map(|x| indexes.at(x)).collect::<Vec<&ModelIndex>>();
                sort_indexes_visually(&mut indexes_sorted, table_view);
                let indexes_sorted = get_real_indexes(&indexes_sorted, filter_model);

                // Create a string to keep all the values in a TSV format (x\tx\tx) and populate it.
                let mut copy = String::new();
                let mut row = 0;
                for (cycle, model_index) in indexes_sorted.iter().enumerate() {
                    if model_index.is_valid() {

                        // If this is the first time we loop, get the row. Otherwise, Replace the last \t with a \n and update the row.
                        if cycle == 0 { row = model_index.row(); }
                        else if model_index.row() != row {
                            copy.pop();
                            copy.push('\n');
                            row = model_index.row();
                        }

                        // If it's checkable, we need to get a bool. Otherwise it's a String.
                        let item = unsafe { model.as_mut().unwrap().item_from_index(&model_index) };
                        if unsafe { item.as_mut().unwrap().is_checkable() } {
                            match unsafe { item.as_mut().unwrap().check_state() } {
                                CheckState::Checked => copy.push_str("true"),
                                CheckState::Unchecked => copy.push_str("false"),
                                _ => return
                            }
                        }
                        else { copy.push_str(&QString::to_std_string(unsafe { &item.as_mut().unwrap().text() })); }

                        // Add a \t to separate fields except if it's the last field.
                        if cycle < (indexes_sorted.len() - 1) { copy.push('\t'); }
                    }
                }

                // Put the baby into the oven.
                unsafe { GuiApplication::clipboard().as_mut().unwrap().set_text(&QString::from_std_str(copy)); }
            }),

            slot_context_menu_copy_as_lua_table: SlotBool::new(clone!(
                table_definition,
                table_type => move |_| {

                    let table_type = &*table_type.borrow();
                    let packed_file_data = match table_type {
                        TableType::DependencyManager(data) => &data,
                        TableType::DB(data) => &data.entries,
                        TableType::LOC(data) => &data.entries,
                    };

                    // Get all the rows into a Vec<Vec<String>>, so we can deal with them more easely.
                    let mut entries = vec![];
                    for row in packed_file_data.iter() {
                        let mut row_string = vec![];
                        for cell in row.iter() {

                            // Get the data of the cell as a String.
                            let cell_data = match cell {
                                DecodedData::Boolean(ref data) => if *data { "true".to_owned() } else { "false".to_owned() },

                                // Floats need to be tweaked to fix trailing zeroes and precission issues, like turning 0.5000004 into 0.5.
                                DecodedData::Float(ref data) => {
                                    let data_str = format!("{}", data);

                                    // If we have more than 3 decimals, we limit it to three, then do magic to remove trailing zeroes.
                                    if let Some(position) = data_str.find('.') {
                                        let decimals = &data_str[position..].len();
                                        if *decimals > 3 { format!("{}", format!("{:.3}", data).parse::<f32>().unwrap()) }
                                        else { data_str }
                                    }
                                    else { data_str }
                                },
                                DecodedData::Integer(ref data) => format!("{}", data),
                                DecodedData::LongInteger(ref data) => format!("{}", data),

                                // All these are Strings, so they need to escape certain chars and include commas in Lua.
                                DecodedData::StringU8(ref data) |
                                DecodedData::StringU16(ref data) |
                                DecodedData::OptionalStringU8(ref data) |
                                DecodedData::OptionalStringU16(ref data) => format!("\"{}\"", data.replace('\\', "\\\\").replace('\"', "\\\"")),
                            };

                            // And push it to the list.
                            row_string.push(cell_data);
                        }
                        entries.push(row_string);
                    }

                    // Get the titles of the columns.
                    let mut column_names = table_definition.fields.iter().map(|x| x.field_name.to_owned()).collect::<Vec<String>>();

                    // Try to get the Key column number if exists and it doesn't have duplicates.
                    let key = 
                        if let Some(column) = table_definition.fields.iter().position(|x| x.field_is_key) {
                            let key_column = entries.iter().map(|x| x[column].to_owned()).collect::<Vec<String>>();
                            let mut key_column_sorted = key_column.to_vec();
                            key_column_sorted.sort();
                            key_column_sorted.dedup();
                            if key_column.len() == key_column_sorted.len() {
                                Some(key_column)
                            }
                            else { None }
                        }

                        // Otherwise, we return a None.
                        else { None };

                    // Reorder the entries to get the same column layout as we visually have in the table.
                    let mut key_columns = vec![];

                    // For each column, if the field is key, add that column to the "Key" list, so we can move them at the begining later.
                    for (index, field) in table_definition.fields.iter().enumerate() {
                        if field.field_is_key { key_columns.push(index); }
                    }

                    // If we have any "Key" field...
                    if !key_columns.is_empty() {

                        // For each key column, move the column to the begining.
                        for (position, column) in key_columns.iter().enumerate() {

                            // We need to do it to the column name list too.
                            let key = column_names.remove(*column);
                            column_names.insert(position, key);

                            for row in &mut entries {
                                let key = row.remove(*column);
                                row.insert(position, key);
                            }
                        }
                    }

                    // Create the string of the table.
                    let mut lua_table = String::new();

                    // If we have a "Key" field, we form a "Map<String, Map<String, Any>>". If we don't have it, we form a "Vector<Map<String, Any>>".
                    match key {
                        Some(column) => {

                            // Start the table.
                            lua_table.push_str("TABLE = {\n");
                            for (index, row) in entries.iter().enumerate() {

                                // Add the "key" of the lua table.
                                lua_table.push_str(&format!("\t[{}] = {{", column[index].to_owned()));

                                // For each cell in the row, push it to the LUA Table.
                                for (column, cell) in row.iter().enumerate() {
                                    lua_table.push_str(&format!(" [\"{}\"] = {},", column_names[column], cell));
                                }

                                // Take out the last comma.
                                lua_table.pop();

                                // Close the row.
                                if index == entries.len() - 1 { lua_table.push_str(" }\n"); }
                                else { lua_table.push_str(" },\n"); }
                            }

                            // Close the table.
                            lua_table.push_str("}");
                        }
                        
                        None => {
                            for (index, row) in entries.iter().enumerate() {
                                lua_table.push('{');
                                for (column, cell) in row.iter().enumerate() {
                                    lua_table.push_str(&format!(" [\"{}\"] = {},", column_names[column], cell));
                                }

                                // Delete the last comma.
                                lua_table.pop();

                                // Close the row.
                                if index == entries.len() - 1 { lua_table.push_str(" }\n"); }
                                else { lua_table.push_str(" },\n"); }
                            }
                        }
                    }

                    // Put the baby into the oven.
                    unsafe { GuiApplication::clipboard().as_mut().unwrap().set_text(&QString::from_std_str(lua_table)); }
                }
            )),

            // NOTE: Saving is not needed in this slot, as this gets detected by the main saving slot.
            slot_context_menu_paste: SlotBool::new(clone!(
                undo_lock,
                save_lock,
                packed_file_path,
                table_state_data,
                table_definition => move |_| {

                    // Get the current selection.
                    let clipboard = GuiApplication::clipboard();
                    let mut text = unsafe { clipboard.as_mut().unwrap().text(()).to_std_string() };

                    // Get the current selection and his, visually speaking, first item (top-left).
                    let indexes = unsafe { table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection().indexes() };
                    let mut indexes_sorted_visual = (0..indexes.count(())).map(|x| indexes.at(x)).collect::<Vec<&ModelIndex>>();
                    sort_indexes_visually(&mut indexes_sorted_visual, table_view);
                    let base_index_visual = if !indexes_sorted_visual.is_empty() { &indexes_sorted_visual[0] } else { return };

                    // If the text ends in \n, remove it. Excel things.
                    if text.ends_with('\n') { text.pop(); }
                    let rows = text.split('\n').collect::<Vec<&str>>();
                    let rows = rows.iter().map(|x| x.split('\t').collect::<Vec<&str>>()).collect::<Vec<Vec<&str>>>();

                    // We're going to try and check in square mode. That means, start in the selected cell, then right
                    // until we reach a \n, then return to the initial column. Due to how sorting works, we have to do
                    // a test pass first and get all the real AND VALID indexes, then try to paste on them.
                    let horizontal_header = unsafe { table_view.as_ref().unwrap().horizontal_header().as_ref().unwrap() };
                    let vertical_header = unsafe { table_view.as_ref().unwrap().vertical_header().as_ref().unwrap() };
                    let mut real_cells = vec![];
                    let mut added_rows = 0;
                    let mut visual_row = vertical_header.visual_index(base_index_visual.row());
                    for row in &rows {
                        let mut visual_column = horizontal_header.visual_index(base_index_visual.column());
                        for text in row {

                            // Depending on the column, we try to encode the data in one format or another, or we just skip it.
                            let real_column = horizontal_header.logical_index(visual_column);
                            let mut real_row = vertical_header.logical_index(visual_row);
                            if let Some(field) = table_definition.fields.get(real_column as usize) {
                                let is_valid_data = match field.field_type {
                                    FieldType::Boolean => if text.to_lowercase() != "true" && text.to_lowercase() != "false" && text != &"1" && text != &"0" { false } else { true },
                                    FieldType::Float => if text.parse::<f32>().is_err() { false } else { true },
                                    FieldType::Integer => if text.parse::<i32>().is_err() { false } else { true },
                                    FieldType::LongInteger => if text.parse::<i64>().is_err() { false } else { true },

                                    // All these are Strings, so we can skip their checks....
                                    FieldType::StringU8 |
                                    FieldType::StringU16 |
                                    FieldType::OptionalStringU8 |
                                    FieldType::OptionalStringU16 => true,
                                };
                                if is_valid_data {
        
                                    // If real_row is -1 (invalid), then we need to add an empty row to the model (NOT TO THE FILTER) and try again.
                                    if real_row == -1 {
                                        let rows = create_empty_rows(&table_definition, 1);
                                        unsafe { model.as_mut().unwrap().append_row(&rows[0]); }
                                        real_row = unsafe { model.as_ref().unwrap().row_count(()) - 1 };
                                        added_rows += 1;
                                    }
                                    real_cells.push((unsafe { filter_model.as_mut().unwrap().map_to_source(&filter_model.as_mut().unwrap().index((real_row, real_column))) }, text));
                                }
                            }
                            visual_column += 1;
                        }
                        visual_row += 1;
                    }

                    // We need to update the undo model here, because otherwise it'll start triggering crashes 
                    // in case the first thing to paste is equal to the current value. In that case, the set_data
                    // will not trigger, and the update_undo_model will not trigger either, causing a crash if 
                    // inmediatly after that we try to paste something in a new line (which will not exist in the undo model).
                    {
                        let mut table_state_data = table_state_data.borrow_mut();
                        let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                        update_undo_model(model, table_state_data.undo_model);
                    }
                    *save_lock.borrow_mut() = true;

                    // Now we do the real pass, changing data if needed.
                    let mut changed_cells = 0;
                    for (index, (real_cell, text)) in real_cells.iter().enumerate() {

                        // Depending on the column, we try to encode the data in one format or another.
                        match table_definition.fields[real_cell.column() as usize].field_type {

                            FieldType::Boolean => {
                                let current_value = unsafe { model.as_ref().unwrap().item_from_index(real_cell).as_ref().unwrap().check_state() };
                                let new_value = if text.to_lowercase() == "true" || **text == "1" { CheckState::Checked } else { CheckState::Unchecked };
                                if current_value != new_value { 
                                    unsafe { model.as_mut().unwrap().item_from_index(real_cell).as_mut().unwrap().set_check_state(new_value); }
                                    changed_cells += 1;
                                }
                            },

                            FieldType::Float => {
                                let current_value = unsafe { model.as_ref().unwrap().data(real_cell).to_string().to_std_string() };
                                if &current_value != *text {
                                    unsafe { model.as_mut().unwrap().set_data((real_cell, &Variant::new2(text.parse::<f32>().unwrap()), 2)); }
                                    changed_cells += 1;
                                }
                            },

                            FieldType::Integer => {
                                let current_value = unsafe { model.as_ref().unwrap().data(real_cell).to_string().to_std_string() };
                                if &current_value != *text {
                                    unsafe { model.as_mut().unwrap().set_data((real_cell, &Variant::new0(text.parse::<i32>().unwrap()), 2)); }
                                    changed_cells += 1;
                                }
                            },

                            FieldType::LongInteger => {
                                let current_value = unsafe { model.as_ref().unwrap().data(real_cell).to_string().to_std_string() };
                                if &current_value != *text {
                                    unsafe { model.as_mut().unwrap().set_data((real_cell, &Variant::new2(text.parse::<i64>().unwrap()), 2)); }
                                    changed_cells += 1;
                                }
                            },

                            _ => {
                                let current_value = unsafe { model.as_ref().unwrap().data(real_cell).to_string().to_std_string() };
                                if &current_value != *text {
                                    unsafe { model.as_mut().unwrap().set_data((real_cell, &Variant::new0(&QString::from_std_str(text)), 2)); }
                                    changed_cells += 1;
                                }
                            }
                        }

                        // If it's the last cycle, trigger a save. That way we ensure a save it's done at the end.
                        if index == real_cells.len() - 1 {
                            *undo_lock.borrow_mut() = true;
                            unsafe { model.as_mut().unwrap().item_from_index(real_cell).as_mut().unwrap().set_data((&Variant::new0(1i32), 16)); }
                            *save_lock.borrow_mut() = false;
                            unsafe { model.as_mut().unwrap().item_from_index(real_cell).as_mut().unwrap().set_data((&Variant::new0(()), 16)); }
                            *undo_lock.borrow_mut() = false;
                        }
                    }

                    // Fix the undo history to have all the previous changed merged into one. Or that's what I wanted.
                    // Sadly, the world doesn't work like that. As we can edit AND add rows, we have to use a combined undo operation.
                    // I'll call it... Carolina.
                    if changed_cells > 0 || added_rows > 0 {
                        {
                            let mut table_state_data = table_state_data.borrow_mut();
                            let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                            let len = table_state_data.undo_history.len();
                            let mut carolina = vec![];
                            if changed_cells > 0 {
                                let mut edits_data = vec![];
                                let mut edits = table_state_data.undo_history.drain((len - changed_cells)..);
                                for edit in &mut edits { if let TableOperations::Editing(mut edit) = edit { edits_data.append(&mut edit); }}
                                carolina.push(TableOperations::Editing(edits_data));
                            }

                            if added_rows > 0 {
                                let mut rows = vec![];
                                unsafe { ((model.as_mut().unwrap().row_count(()) - added_rows)..model.as_mut().unwrap().row_count(())).rev().for_each(|x| rows.push(x)); }
                                carolina.push(TableOperations::AddRows(rows));
                            }

                            table_state_data.undo_history.push(TableOperations::Carolina(carolina));
                            table_state_data.redo_history.clear();
                            update_undo_model(model, table_state_data.undo_model); 
                        }
                        unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }                           
                    }
                }
            )),

            slot_context_menu_paste_as_new_lines: SlotBool::new(clone!(
                global_search_explicit_paths,
                dependency_data,
                packed_file_path,
                app_ui,
                table_definition,
                table_state_data,
                table_type,
                receiver_qt,
                sender_qt,
                sender_qt_data => move |_| {

                    // If whatever it's in the Clipboard is pasteable in our selection...
                    if Self::check_clipboard_append_rows(table_view, &table_definition) {

                        // Get the text from the clipboard.
                        let clipboard = GuiApplication::clipboard();
                        let mut text = unsafe { clipboard.as_mut().unwrap().text(()).to_std_string() };

                        // If the text ends in \n, remove it. Excel things. We don't use newlines, so replace them with '\t'.
                        if text.ends_with('\n') { text.pop(); }
                        let text = text.replace('\n', "\t");
                        let text = text.split('\t').collect::<Vec<&str>>();

                        // Create a new list of StandardItem, ready to be populated.
                        let mut column = 0;
                        let mut qlist_unordered = vec![];
                        let mut qlist = ListStandardItemMutPtr::new(());
                        for cell in &text {
                            let column_logical_index = unsafe { table_view.as_ref().unwrap().horizontal_header().as_ref().unwrap().logical_index(column) };
                            let field = &table_definition.fields[column_logical_index as usize];
                            let mut item = StandardItem::new(());

                            // Depending on the column, we populate the cell with one thing or another.
                            match &field.field_type {

                                // If its a boolean, prepare it as a boolean.
                                FieldType::Boolean => {
                                    item.set_editable(false);
                                    item.set_checkable(true);
                                    item.set_check_state(if cell.to_lowercase() == "true" || *cell == "1" { CheckState::Checked } else { CheckState::Unchecked });
                                    item.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                },
                                
                                FieldType::Float => {
                                    item.set_data((&Variant::new2(cell.parse::<f32>().unwrap()), 2));
                                    item.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                },

                                FieldType::Integer => {
                                    item.set_data((&Variant::new0(cell.parse::<i32>().unwrap()), 2));
                                    item.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                },

                                FieldType::LongInteger => {
                                    item.set_data((&Variant::new2(cell.parse::<i64>().unwrap()), 2));
                                    item.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                },

                                // In any other case, we treat it as a string. Type-checking is done before this and while saving.
                                _ => {
                                    item.set_text(&QString::from_std_str(cell));
                                    item.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                }
                            }

                            // If we have the dependency stuff enabled, check if it's a valid reference.
                            if SETTINGS.lock().unwrap().settings_bool["use_dependency_checker"] && field.field_is_reference.is_some() {
                                Self::check_references(&dependency_data, column as i32, item.as_mut_ptr());
                            }

                            // Add the cell to the list.
                            qlist_unordered.push((column_logical_index, item.into_raw()));

                            // If we are in the last column, append the list to the Table and reset it.
                            if column as usize == &table_definition.fields.len() - 1 {
                                qlist_unordered.sort_unstable_by_key(|x| x.0);
                                for (_, item) in &qlist_unordered { unsafe { qlist.append_unsafe(&item.clone()); }}
    
                                unsafe { model.as_mut().unwrap().append_row(&qlist); }
                                qlist = ListStandardItemMutPtr::new(());
                                qlist_unordered.clear();
                                column = 0;
                            }

                            // Otherwise, increase the column count.
                            else { column += 1; }
                        }

                        // If the last list was incomplete...
                        if column != 0 {

                            // For each columns we lack...
                            for column in column..table_definition.fields.len() as i32 {

                                // Get the new field.
                                let column_logical_index = unsafe { table_view.as_ref().unwrap().horizontal_header().as_ref().unwrap().logical_index(column) };
                                let field = &table_definition.fields[column_logical_index as usize];

                                // Create a new Item.
                                let mut item = match field.field_type {

                                    // This one needs a couple of changes before turning it into an item in the table.
                                    FieldType::Boolean => {
                                        let mut item = StandardItem::new(());
                                        item.set_editable(false);
                                        item.set_checkable(true);
                                        item.set_check_state(CheckState::Checked);
                                        item
                                    }

                                    FieldType::Float => {
                                        let mut item = StandardItem::new(());
                                        item.set_data((&Variant::new2(0.0f32), 2));
                                        item
                                    },

                                    FieldType::Integer => {
                                        let mut item = StandardItem::new(());
                                        item.set_data((&Variant::new0(0i32), 2));
                                        item
                                    },
                                    
                                    FieldType::LongInteger => {
                                        let mut item = StandardItem::new(());
                                        item.set_data((&Variant::new2(0i64), 2));
                                        item
                                    },

                                    // All these are Strings, so it can be together.
                                    FieldType::StringU8 |
                                    FieldType::StringU16 |
                                    FieldType::OptionalStringU8 |
                                    FieldType::OptionalStringU16 => StandardItem::new(&QString::from_std_str("")),
                                };

                                item.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                qlist_unordered.push((column_logical_index, item.into_raw()));
                            }

                            // Append the list to the Table.
                            qlist_unordered.sort_unstable_by_key(|x| x.0);
                            for (_, item) in &qlist_unordered { unsafe { qlist.append_unsafe(&item.clone()); }}
                            unsafe { model.as_mut().unwrap().append_row(&qlist); }
                        }

                        // If we pasted something, try to save the PackedFile to the main PackFile.
                        if !text.is_empty() {
                            Self::save_to_packed_file(
                                &sender_qt,
                                &sender_qt_data,
                                &receiver_qt,
                                &app_ui,
                                &packed_file_path,
                                model,
                                &global_search_explicit_paths,
                                update_global_search_stuff,
                                &table_definition,
                                &mut table_type.borrow_mut(),
                            );

                            // Update the search stuff, if needed.
                            unsafe { update_search_stuff.as_mut().unwrap().trigger(); }

                            // Update the undo stuff.
                            {
                                let mut table_state_data = table_state_data.borrow_mut();
                                let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                                let mut rows = vec![];
                                unsafe { (table_state_data.undo_model.as_mut().unwrap().row_count(())..model.as_mut().unwrap().row_count(())).rev().for_each(|x| rows.push(x)); }
                                table_state_data.undo_history.push(TableOperations::AddRows(rows));
                                table_state_data.redo_history.clear();
                                update_undo_model(model, table_state_data.undo_model); 
                            }

                            unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                        }
                    }
                }
            )),

            slot_context_menu_paste_to_fill_selection: SlotBool::new(clone!(
                packed_file_path,
                table_state_data,
                table_definition => move |_| {

                    // If whatever it's in the Clipboard is pasteable in our selection...
                    if Self::check_clipboard_to_fill_selection(&table_definition, table_view, model, filter_model) {

                        // Get the text from the clipboard and the list of cells to paste to.
                        let clipboard = GuiApplication::clipboard();
                        let text = unsafe { clipboard.as_mut().unwrap().text(()).to_std_string() };
                        let indexes = unsafe { filter_model.as_mut().unwrap().map_selection_to_source(&table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };

                        let mut changed_cells = 0;
                        for index in 0..indexes.count(()) {
                            let model_index = indexes.at(index);
                            if model_index.is_valid() {

                                // Get the column of that cell.
                                let column = model_index.column();
                                let item = unsafe { model.as_mut().unwrap().item_from_index(&model_index) };

                                // Depending on the column, we try to encode the data in one format or another.
                                match table_definition.fields[column as usize].field_type {
                                    FieldType::Boolean => {
                                        let current_value = unsafe { item.as_mut().unwrap().check_state() };
                                        let new_value = if text.to_lowercase() == "true" || text == "1" { CheckState::Checked } else { CheckState::Unchecked };
                                        if current_value != new_value { 
                                            unsafe { item.as_mut().unwrap().set_check_state(new_value); }
                                            changed_cells += 1;
                                        }
                                    },

                                    FieldType::Float => {
                                        let current_value = unsafe { item.as_mut().unwrap().text().to_std_string() };
                                        if *current_value != text {
                                            unsafe { item.as_mut().unwrap().set_data((&Variant::new2(text.parse::<f32>().unwrap()), 2)); }
                                            changed_cells += 1;
                                        }
                                    },

                                    FieldType::Integer => {
                                        let current_value = unsafe { item.as_mut().unwrap().text().to_std_string() };
                                        if *current_value != text {
                                            unsafe { item.as_mut().unwrap().set_data((&Variant::new0(text.parse::<i32>().unwrap()), 2)); }
                                            changed_cells += 1;
                                        }
                                    },

                                    FieldType::LongInteger => {
                                        let current_value = unsafe { item.as_mut().unwrap().text().to_std_string() };
                                        if *current_value != text {
                                            unsafe { item.as_mut().unwrap().set_data((&Variant::new2(text.parse::<i64>().unwrap()), 2)); }
                                            changed_cells += 1;
                                        }
                                    },

                                    _ => {
                                        let current_value = unsafe { item.as_mut().unwrap().text().to_std_string() };
                                        if *current_value != text {
                                            unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str(&text)); }
                                            changed_cells += 1;
                                        }
                                    }
                                }
                            }
                        }

                        // Fix the undo history to have all the previous changed merged into one.
                        if changed_cells > 0 {
                            {
                                let mut table_state_data = table_state_data.borrow_mut();
                                let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                                let len = table_state_data.undo_history.len();
                                let mut edits_data = vec![];
                                {
                                    let mut edits = table_state_data.undo_history.drain((len - changed_cells)..);
                                    for edit in &mut edits { if let TableOperations::Editing(mut edit) = edit { edits_data.append(&mut edit); }}
                                }

                                table_state_data.undo_history.push(TableOperations::Editing(edits_data));
                                table_state_data.redo_history.clear();
                                update_undo_model(model, table_state_data.undo_model); 
                            }
                            unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }                           
                        }
                    }
                }
            )),

            slot_context_menu_selection_invert: SlotBool::new(move |_| {
                let rows = unsafe { filter_model.as_mut().unwrap().row_count(()) };
                let columns = unsafe { filter_model.as_mut().unwrap().column_count(()) };
                if rows > 0 && columns > 0 {
                    let selection_model = unsafe { table_view.as_mut().unwrap().selection_model() };
                    let first_item = unsafe { filter_model.as_mut().unwrap().index((0, 0)) };
                    let last_item = unsafe { filter_model.as_mut().unwrap().index((rows - 1, columns - 1)) } ;
                    let selection = ItemSelection::new((&first_item, &last_item));
                    unsafe { selection_model.as_mut().unwrap().select((&selection, Flags::from_enum(SelectionFlag::Toggle))); }
                }
            }),

            slot_context_menu_sidebar: SlotBool::new(move |_| {
                unsafe {
                    if sidebar_scroll_area.as_mut().unwrap().is_visible() { sidebar_scroll_area.as_mut().unwrap().hide(); } 
                    else { sidebar_scroll_area.as_mut().unwrap().show(); }
                }
            }),

            slot_context_menu_search: SlotBool::new(move |_| {
                unsafe {
                    if search_widget.as_mut().unwrap().is_visible() { search_widget.as_mut().unwrap().hide(); } 
                    else { search_widget.as_mut().unwrap().show(); }
                }
            }),

            slot_context_menu_import: SlotBool::new(clone!(
                global_search_explicit_paths,
                app_ui,
                table_definition,
                packed_file_path,
                table_state_data,
                table_type,
                enable_header_popups,
                sender_qt,
                sender_qt_data,
                receiver_qt,
                dependency_data,
                receiver_qt => move |_| {
                    
                    // Create the FileDialog to import the TSV file and configure it.
                    let mut file_dialog = unsafe { FileDialog::new_unsafe((
                        app_ui.window as *mut Widget,
                        &QString::from_std_str("Select TSV File to Import..."),
                    )) };

                    file_dialog.set_name_filter(&QString::from_std_str("TSV Files (*.tsv)"));

                    // Run it and, if we receive 1 (Accept), try to import the TSV file.
                    if file_dialog.exec() == 1 {
                        let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());
                        let (name, version, old_data) = match &*table_type.borrow() {
                            TableType::DependencyManager(data) => (TSV_HEADER_PACKFILE_LIST.to_owned(), 1, data.to_vec()),
                            TableType::DB(data) => (data.db_type.to_owned(), data.version, data.entries.to_vec()),
                            TableType::LOC(data) => (TSV_HEADER_LOC_PACKEDFILE.to_owned(), 1, data.entries.to_vec()),
                        };

                        sender_qt.send(Commands::ImportTSVPackedFile).unwrap();
                        sender_qt_data.send(Data::DefinitionPathBufStringI32(((*table_definition).clone(), path, name, version))).unwrap();

                        match check_message_validity_recv2(&receiver_qt) {
                            Data::VecVecDecodedData(new_data) => {
                                match &mut *table_type.borrow_mut() {
                                    TableType::DependencyManager(data) => *data = new_data.to_vec(),
                                    TableType::DB(data) => data.entries = new_data.to_vec(),
                                    TableType::LOC(data) => data.entries = new_data.to_vec(),
                                };
                                Self::load_data_to_table_view(table_view, model, &table_type.borrow(), &table_definition, &dependency_data)
                            },
                            Data::Error(error) => return show_dialog(app_ui.window, false, error),
                            _ => panic!(THREADS_MESSAGE_ERROR),
                        }

                        // Build the Column's "Data".
                        Self::build_columns(table_view, table_view_frozen, model, &table_definition, enable_header_popups.clone());

                        if SETTINGS.lock().unwrap().settings_bool["adjust_columns_to_content"] {
                            unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
                        }

                        // Try to save the PackedFile to the main PackFile.
                        Self::save_to_packed_file(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &app_ui,
                            &packed_file_path,
                            model,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
                            &table_definition,
                            &mut table_type.borrow_mut(),
                        );

                        // Update the search stuff, if needed.
                        unsafe { update_search_stuff.as_mut().unwrap().trigger(); }

                        {
                            let mut table_state_data = table_state_data.borrow_mut();
                            let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                            table_state_data.undo_history.push(TableOperations::ImportTSV(old_data));
                            table_state_data.redo_history.clear();
                            update_undo_model(model, table_state_data.undo_model); 
                        }
                        unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                    }
                }
            )),
            slot_context_menu_export: SlotBool::new(clone!(
                table_definition,
                table_type,
                app_ui,
                sender_qt,
                sender_qt_data,
                receiver_qt => move |_| {

                    // Create a File Chooser to get the destination path and configure it.
                    let mut file_dialog = unsafe { FileDialog::new_unsafe((
                        app_ui.window as *mut Widget,
                        &QString::from_std_str("Export TSV File..."),
                    )) };

                    file_dialog.set_accept_mode(qt_widgets::file_dialog::AcceptMode::Save);
                    file_dialog.set_confirm_overwrite(true);
                    file_dialog.set_name_filter(&QString::from_std_str("TSV Files (*.tsv)"));
                    file_dialog.set_default_suffix(&QString::from_std_str("tsv"));

                    // Run it and, if we receive 1 (Accept), export the DB Table.
                    if file_dialog.exec() == 1 {

                        let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());
                        let headers = table_definition.fields.iter().map(|x| x.field_name.to_owned()).collect::<Vec<String>>();
                        let (name, version, entries) = match &*table_type.borrow() {
                            TableType::DependencyManager(data) => (TSV_HEADER_PACKFILE_LIST.to_owned(), 1, data.to_vec()),
                            TableType::DB(data) => (data.db_type.to_owned(), data.version, data.entries.to_vec()),
                            TableType::LOC(data) => (TSV_HEADER_LOC_PACKEDFILE.to_owned(), 1, data.entries.to_vec()),
                        };

                        sender_qt.send(Commands::ExportTSVPackedFile).unwrap();
                        sender_qt_data.send(Data::VecVecDecodedDataPathBufVecStringTupleStrI32((entries.to_vec(), path, headers, (name, version)))).unwrap();

                        // If there is an error, report it. Otherwise, we're done.
                        match check_message_validity_recv2(&receiver_qt) {
                            Data::Success => return,
                            Data::Error(error) => return show_dialog(app_ui.window, false, error),
                            _ => panic!(THREADS_MESSAGE_ERROR),
                        }
                    }
                }
            )),

            slot_smart_delete: SlotBool::new(clone!(
                global_search_explicit_paths,
                app_ui,
                table_definition,
                packed_file_path,
                table_state_data,
                table_type,
                receiver_qt,
                sender_qt,
                sender_qt_data => move |_| {

                    // Get all the cells selected, separated by rows.
                    let indexes = unsafe { filter_model.as_mut().unwrap().map_selection_to_source(&table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };
                    let mut cells: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
                    for index in 0..indexes.size() {
                        let model_index = indexes.at(index);
                        if model_index.is_valid() {

                            // Get the current row and column.
                            let row = model_index.row();
                            let column = model_index.column();

                            // Check if we have any cell in that row and add/insert the new one.
                            let mut x = false;
                            match cells.get_mut(&row) {
                                Some(cells) => cells.push(column),
                                None => { x = true },
                            }
                            if x { cells.insert(row, vec![column]); }
                        }
                    }

                    // First, we do all the edits needed.
                    let mut edits = vec![];
                    for (key, values) in cells.iter() {
                        if values.len() < unsafe { model.as_ref().unwrap().column_count(()) as usize } {
                            for column in values {
                                let item = unsafe { model.as_mut().unwrap().item((*key, *column)) };
                                match table_definition.fields[*column as usize].field_type {
                                    FieldType::Boolean => {
                                        let current_value = unsafe { item.as_mut().unwrap().check_state() };
                                        if current_value != CheckState::Unchecked { 
                                            unsafe { edits.push(((*key, *column), (&*item).clone())); }
                                            unsafe { item.as_mut().unwrap().set_check_state(CheckState::Unchecked); }
                                        }
                                    }

                                    FieldType::Float => {
                                        let current_value = unsafe { item.as_mut().unwrap().text().to_std_string() };
                                        if !current_value.is_empty() {
                                            unsafe { edits.push(((*key, *column), (&*item).clone())); }
                                            unsafe { item.as_mut().unwrap().set_data((&Variant::new2(0.0f32), 2)); }
                                        }
                                    }

                                    FieldType::Integer => {
                                        let current_value = unsafe { item.as_mut().unwrap().text().to_std_string() };
                                        if !current_value.is_empty() {
                                            unsafe { edits.push(((*key, *column), (&*item).clone())); }
                                            unsafe { item.as_mut().unwrap().set_data((&Variant::new0(0i32), 2)); }
                                        }
                                    }

                                    FieldType::LongInteger => {
                                        let current_value = unsafe { item.as_mut().unwrap().text().to_std_string() };
                                        if !current_value.is_empty() {
                                            unsafe { edits.push(((*key, *column), (&*item).clone())); }
                                            unsafe { item.as_mut().unwrap().set_data((&Variant::new2(0i64), 2)); }
                                        }
                                    }

                                    _ => {
                                        let current_value = unsafe { item.as_mut().unwrap().text().to_std_string() };
                                        if !current_value.is_empty() {
                                            unsafe { edits.push(((*key, *column), (&*item).clone())); }
                                            unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str("")); }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Then, we delete all the fully selected rows. This time in reverse.
                    let mut removed_rows_splitted = vec![];
                    let mut current_row_pack = vec![];
                    let mut current_row_index = -2;
                    for (index, (row, _)) in cells.iter().rev().filter(|x| x.1.len() == unsafe { model.as_ref().unwrap().column_count(()) as usize }).enumerate() {

                        let mut items = vec![];                        
                        for column in 0..unsafe { model.as_mut().unwrap().column_count(()) } {
                            let item = unsafe { &*model.as_mut().unwrap().item((*row, column)) };
                            items.push(item.clone());
                        }

                        if (*row == current_row_index - 1) || index == 0 { 
                            current_row_pack.push((*row, items)); 
                            current_row_index = *row;
                        }
                        else {
                            current_row_pack.reverse();
                            removed_rows_splitted.push(current_row_pack.to_vec());
                            current_row_pack.clear();
                            current_row_pack.push((*row, items)); 
                            current_row_index = *row;
                        }
                    }
                    
                    current_row_pack.reverse();
                    removed_rows_splitted.push(current_row_pack);
                    if removed_rows_splitted[0].is_empty() { removed_rows_splitted.clear(); }

                    for row_pack in removed_rows_splitted.iter() {
                        unsafe { model.as_mut().unwrap().remove_rows((row_pack[0].0, row_pack.len() as i32)); }
                    }

                    // When you delete a row, the save has to be triggered manually. For cell edits it get's triggered automatically.
                    if !cells.is_empty() {
                        Self::save_to_packed_file(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &app_ui,
                            &packed_file_path,
                            model,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
                            &table_definition,
                            &mut table_type.borrow_mut(),
                        );

                        // Update the search stuff, if needed.
                        unsafe { update_search_stuff.as_mut().unwrap().trigger(); }

                         {
                            // Update the undo stuff. This is a bit special, as we have to remove all the automatically created "Editing" undos.
                            let mut table_state_data = table_state_data.borrow_mut();
                            let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                            let len = table_state_data.undo_history.len();
                            table_state_data.undo_history.truncate(len - edits.len());
                            removed_rows_splitted.reverse();
                            table_state_data.undo_history.push(TableOperations::SmartDelete((edits, removed_rows_splitted)));
                            table_state_data.redo_history.clear();
                            update_undo_model(model, table_state_data.undo_model); 
                        }

                        unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }                        
                    }
                }
            )),

            // This is the list of slots to toggle things in columns. Is created before all this, so here we just add it.
            slots_hide_show_column,
            slots_freeze_unfreeze_column,

            // Slot to close the search widget.
            slot_update_search_stuff: SlotNoArgs::new(clone!(
                matches,
                position,
                table_definition,
                packed_file_path,
                search_data => move || {

                    // Get all the stuff separated, to make it clear.
                    let text = &search_data.borrow().0;
                    let flags = &search_data.borrow().1;
                    let column = search_data.borrow().2;

                    // If there is no text or we don't have the search bar open, return emptyhanded.
                    unsafe { if text.is_empty() || !search_widget.as_mut().unwrap().is_visible() { return }}

                    // Reset the matches's list.
                    matches.borrow_mut().clear();
                    
                    // Get the column selected, and act depending on it.
                    match column {
                        -1 => {

                            // Get all the matches from all the columns.
                            for index in 0..table_definition.fields.len() {
                                let matches_unprocessed = unsafe { model.as_mut().unwrap().find_items((&QString::from_std_str(text), flags.clone(), index as i32)) };
                                for index in 0..matches_unprocessed.count() {
                                    let model_index = unsafe { matches_unprocessed.at(index).as_mut().unwrap().index() };
                                    let filter_model_index = unsafe { filter_model.as_mut().unwrap().map_from_source(&model_index) };
                                    matches.borrow_mut().insert(
                                        ModelIndexWrapped::new(model_index),
                                        if filter_model_index.is_valid() { Some(ModelIndexWrapped::new(filter_model_index)) } else { None }
                                    );
                                }
                            }
                        },

                        _ => {

                            // Once you got them, process them and get their ModelIndex.
                            let matches_unprocessed = unsafe { model.as_mut().unwrap().find_items((&QString::from_std_str(text), flags.clone(), column)) };
                            for index in 0..matches_unprocessed.count() {
                                let model_index = unsafe { matches_unprocessed.at(index).as_mut().unwrap().index() };
                                let filter_model_index = unsafe { filter_model.as_mut().unwrap().map_from_source(&model_index) };
                                matches.borrow_mut().insert(
                                    ModelIndexWrapped::new(model_index),
                                    if filter_model_index.is_valid() { Some(ModelIndexWrapped::new(filter_model_index)) } else { None }
                                );
                            }
                        }
                    }

                    // If no matches have been found, report it.
                    if matches.borrow().is_empty() {
                        *position.borrow_mut() = None;
                        unsafe { matches_label.as_mut().unwrap().set_text(&QString::from_std_str("No matches found.")); }
                        unsafe { prev_match_button.as_mut().unwrap().set_enabled(false); }
                        unsafe { next_match_button.as_mut().unwrap().set_enabled(false); }
                        unsafe { replace_current_button.as_mut().unwrap().set_enabled(false); }
                        unsafe { replace_all_button.as_mut().unwrap().set_enabled(false); }
                    }

                    // Otherwise, get the matches in the filter and check them.
                    else {
                        let matches_in_filter = matches.borrow().iter().filter(|x| x.1.is_some()).count();

                        // If no matches have been found in the current filter, but they have been in the model...
                        if matches_in_filter == 0 {
                            *position.borrow_mut() = None;
                            unsafe { matches_label.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{} with current filter ({} in total)", matches_in_filter, matches.borrow().len()))); }
                            unsafe { prev_match_button.as_mut().unwrap().set_enabled(false); }
                            unsafe { next_match_button.as_mut().unwrap().set_enabled(false); }
                            unsafe { replace_current_button.as_mut().unwrap().set_enabled(false); }
                            unsafe { replace_all_button.as_mut().unwrap().set_enabled(false); }
                        }
                        
                        // Otherwise, matches have been found both, in the model and in the filter.
                        else {

                            // Calculate the new position to be selected.
                            let new_position = match *position.borrow() {
                                Some(pos) => {

                                    // Get the list of all valid ModelIndex for the current filter.
                                    let matches = matches.borrow();
                                    let matches_in_filter = matches.iter().filter(|x| x.1.is_some()).map(|x| x.1.as_ref().unwrap().get()).collect::<Vec<&ModelIndex>>();
                                    
                                    // If our position is still valid, use it. Otherwise, return a 0.
                                    if pos < matches_in_filter.len() { pos } else { 0 }
                                }
                                None => 0
                            };

                            *position.borrow_mut() = Some(new_position);
                            unsafe { matches_label.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{} of {} with current filter ({} in total)", position.borrow().unwrap() + 1, matches_in_filter, matches.borrow().len()))); }
                            
                            if position.borrow().unwrap() == 0 { unsafe { prev_match_button.as_mut().unwrap().set_enabled(false); }}
                            else { unsafe { prev_match_button.as_mut().unwrap().set_enabled(true); }}

                            if matches_in_filter > 1 && position.borrow().unwrap() < (matches_in_filter - 1) { unsafe { next_match_button.as_mut().unwrap().set_enabled(true); }}
                            else { unsafe { next_match_button.as_mut().unwrap().set_enabled(false); }}

                            unsafe { replace_current_button.as_mut().unwrap().set_enabled(true); }
                            unsafe { replace_all_button.as_mut().unwrap().set_enabled(true); }
                        }
                    }

                    // Add the new search data to the state history.
                    if let Some(state) = TABLE_STATES_UI.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
                        unsafe { state.search_state = SearchState::new(search_line_edit.as_mut().unwrap().text().to_std_string(), replace_line_edit.as_mut().unwrap().text().to_std_string(), column_selector.as_ref().unwrap().current_index(), case_sensitive_button.as_mut().unwrap().is_checked()); }
                    }
                }
            )),

            // Slot for the search button.
            slot_search: SlotNoArgs::new(clone!(
                matches,
                table_definition,
                packed_file_path,
                position => move || {

                    // Reset the data and get the text.
                    matches.borrow_mut().clear();
                    *position.borrow_mut() = None;
                    let text = unsafe { search_line_edit.as_mut().unwrap().text() };
                    
                    // If there is no text, return emptyhanded.
                    if text.is_empty() { 
                        *search_data.borrow_mut() = (String::new(), Flags::from_enum(MatchFlag::Contains), -1);
                        unsafe { matches_label.as_mut().unwrap().set_text(&QString::from_std_str("")); }
                        unsafe { prev_match_button.as_mut().unwrap().set_enabled(false); }
                        unsafe { next_match_button.as_mut().unwrap().set_enabled(false); }
                        unsafe { replace_current_button.as_mut().unwrap().set_enabled(false); }
                        unsafe { replace_all_button.as_mut().unwrap().set_enabled(false); }
                        return
                    }

                    // Create the flags needed for the search.
                    // TODO: For some reason, if we try to use regexp here, it doesn't find anything. So we need to find out why.
                    let mut flags = Flags::from_enum(MatchFlag::Contains);

                    // Check if the filter should be "Case Sensitive".
                    let case_sensitive = unsafe { case_sensitive_button.as_mut().unwrap().is_checked() };
                    if case_sensitive { flags = flags | Flags::from_enum(MatchFlag::CaseSensitive); }
                    
                    // Get the column selected, and act depending on it.
                    let column = unsafe { column_selector.as_mut().unwrap().current_text().to_std_string().replace(' ', "_").to_lowercase() };
                    match &*column {
                        "*_(all_columns)" => {
                            for index in 0..table_definition.fields.len() {
                                
                                // Get all the matches from all the columns. Once you got them, process them and get their ModelIndex.
                                let matches_unprocessed = unsafe { model.as_mut().unwrap().find_items((&text, flags.clone(), index as i32)) };
                                for index in 0..matches_unprocessed.count() {
                                    let model_index = unsafe { matches_unprocessed.at(index).as_mut().unwrap().index() };
                                    let filter_model_index = unsafe { filter_model.as_mut().unwrap().map_from_source(&model_index) };
                                    matches.borrow_mut().insert(
                                        ModelIndexWrapped::new(model_index),
                                        if filter_model_index.is_valid() { Some(ModelIndexWrapped::new(filter_model_index)) } else { None }
                                    );
                                }
                            }
                        },

                        _ => {
                            let column = table_definition.fields.iter().position(|x| x.field_name == column).unwrap();

                            // Once you got them, process them and get their ModelIndex.
                            let matches_unprocessed = unsafe { model.as_mut().unwrap().find_items((&text, flags.clone(), column as i32)) };
                            for index in 0..matches_unprocessed.count() {
                                let model_index = unsafe { matches_unprocessed.at(index).as_mut().unwrap().index() };
                                let filter_model_index = unsafe { filter_model.as_mut().unwrap().map_from_source(&model_index) };
                                matches.borrow_mut().insert(
                                    ModelIndexWrapped::new(model_index),
                                    if filter_model_index.is_valid() { Some(ModelIndexWrapped::new(filter_model_index)) } else { None }
                                );
                            }
                        }
                    }

                    // If no matches have been found, report it.
                    if matches.borrow().is_empty() {
                        *position.borrow_mut() = None;
                        unsafe { matches_label.as_mut().unwrap().set_text(&QString::from_std_str("No matches found.")); }
                        unsafe { prev_match_button.as_mut().unwrap().set_enabled(false); }
                        unsafe { next_match_button.as_mut().unwrap().set_enabled(false); }
                        unsafe { replace_current_button.as_mut().unwrap().set_enabled(false); }
                        unsafe { replace_all_button.as_mut().unwrap().set_enabled(false); }
                    }

                    // Otherwise...
                    else {

                        let matches_in_filter = matches.borrow().iter().filter(|x| x.1.is_some()).count();

                        // If no matches have been found in the current filter, but they have been in the model...
                        if matches_in_filter == 0 {
                            *position.borrow_mut() = None;
                            unsafe { matches_label.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{} with current filter ({} in total)", matches_in_filter, matches.borrow().len()))); }
                            unsafe { prev_match_button.as_mut().unwrap().set_enabled(false); }
                            unsafe { next_match_button.as_mut().unwrap().set_enabled(false); }
                            unsafe { replace_current_button.as_mut().unwrap().set_enabled(false); }
                            unsafe { replace_all_button.as_mut().unwrap().set_enabled(false); }
                        }
                        
                        // Otherwise, matches have been found both, in the model and in the filter.
                        else {

                            *position.borrow_mut() = Some(0);
                            unsafe { matches_label.as_mut().unwrap().set_text(&QString::from_std_str(&format!("1 of {} with current filter ({} in total)", matches_in_filter, matches.borrow().len()))); }
                            unsafe { prev_match_button.as_mut().unwrap().set_enabled(false); }
                            if matches_in_filter > 1 { unsafe { next_match_button.as_mut().unwrap().set_enabled(true); }}
                            else { unsafe { next_match_button.as_mut().unwrap().set_enabled(false); }}

                            unsafe { replace_current_button.as_mut().unwrap().set_enabled(true); }
                            unsafe { replace_all_button.as_mut().unwrap().set_enabled(true); }

                            let selection_model = unsafe { table_view.as_mut().unwrap().selection_model() };
                            unsafe { selection_model.as_mut().unwrap().select((
                                matches.borrow().iter().find(|x| x.1.is_some()).map(|x| x.1.as_ref().unwrap().get()).unwrap(),
                                Flags::from_enum(SelectionFlag::ClearAndSelect)
                            )); }
                        }
                    }

                    // Add the new search data to the state history.
                    *search_data.borrow_mut() = (text.to_std_string(), flags, table_definition.fields.iter().position(|x| x.field_name == column).map(|x| x as i32).unwrap_or(-1));
                    if let Some(state) = TABLE_STATES_UI.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
                        unsafe { state.search_state = SearchState::new(search_line_edit.as_mut().unwrap().text().to_std_string(), replace_line_edit.as_mut().unwrap().text().to_std_string(), column_selector.as_ref().unwrap().current_index(), case_sensitive_button.as_mut().unwrap().is_checked()); }
                    }
                }
            )),

            // Slots for the prev/next buttons.
            slot_prev_match: SlotNoArgs::new(clone!(
                matches,
                position => move || {

                    // Get the list of all valid ModelIndex for the current filter and the current position.
                    let matches = matches.borrow();
                    let matches_in_filter = matches.iter().filter(|x| x.1.is_some()).map(|x| x.1.as_ref().unwrap().get()).collect::<Vec<&ModelIndex>>();
                    if let Some(ref mut pos) = *position.borrow_mut() { 
                    
                        // If we are in an invalid result, return. If it's the first one, disable the button and return.
                        if *pos > 0 {
                            *pos -= 1;
                            if *pos == 0 { unsafe { prev_match_button.as_mut().unwrap().set_enabled(false); }}
                            else { unsafe { prev_match_button.as_mut().unwrap().set_enabled(true); }}
                            if *pos >= matches_in_filter.len() - 1 { unsafe { next_match_button.as_mut().unwrap().set_enabled(false); }}
                            else { unsafe { next_match_button.as_mut().unwrap().set_enabled(true); }}

                            // Select the new cell.
                            let selection_model = unsafe { table_view.as_mut().unwrap().selection_model() };
                            unsafe { selection_model.as_mut().unwrap().select((
                                matches_in_filter[*pos],
                                Flags::from_enum(SelectionFlag::ClearAndSelect)
                            )); }

                            unsafe { matches_label.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{} of {} with current filter ({} in total)", *pos + 1, matches_in_filter.len(), matches.len()))); }
                        }
                    }
                }
            )),
            slot_next_match: SlotNoArgs::new(clone!(
                matches,
                position => move || {

                    // Get the list of all valid ModelIndex for the current filter and the current position.
                    let matches = matches.borrow();
                    let matches_in_filter = matches.iter().filter(|x| x.1.is_some()).map(|x| x.1.as_ref().unwrap().get()).collect::<Vec<&ModelIndex>>();
                    if let Some(ref mut pos) = *position.borrow_mut() { 
                    
                        // If we are in an invalid result, return. If it's the last one, disable the button and return.
                        if *pos >= matches_in_filter.len() - 1 { unsafe { next_match_button.as_mut().unwrap().set_enabled(false); }}
                        else {
                            *pos += 1;
                            if *pos == 0 { unsafe { prev_match_button.as_mut().unwrap().set_enabled(false); }}
                            else { unsafe { prev_match_button.as_mut().unwrap().set_enabled(true); }}
                            if *pos >= matches_in_filter.len() - 1 { unsafe { next_match_button.as_mut().unwrap().set_enabled(false); }}
                            else { unsafe { next_match_button.as_mut().unwrap().set_enabled(true); }}

                            // Select the new cell.
                            let selection_model = unsafe { table_view.as_mut().unwrap().selection_model() };                            
                            unsafe { selection_model.as_mut().unwrap().select((
                                matches_in_filter[*pos],
                                Flags::from_enum(SelectionFlag::ClearAndSelect)
                            )); }

                            unsafe { matches_label.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{} of {} with current filter ({} in total)", *pos + 1, matches_in_filter.len(), matches.len()))); }
                        }
                    }
                }
            )),

            // Slot to close the search widget.
            slot_close_search: SlotNoArgs::new(move || {
                unsafe { search_widget.as_mut().unwrap().hide(); }
                unsafe { table_view.as_mut().unwrap().set_focus(()); }
            }),

            // Slot for the "Replace Current" button. This triggers the main save function, so we can let that one update the search stuff.
            slot_replace_current: SlotNoArgs::new(clone!(
                table_definition,
                app_ui,
                matches,
                position => move || {
                    
                    // Get the text, and only proceed if the source is not empty.
                    let text_source = unsafe { search_line_edit.as_mut().unwrap().text().to_std_string() };
                    let text_replace = unsafe { replace_line_edit.as_mut().unwrap().text().to_std_string() };
                    if !text_source.is_empty() {

                        // This is done like that because problems with borrowing matches and position. We cannot set the new text while
                        // matches is borrowed, so we have to catch that into his own scope.
                        let item;
                        let replaced_text;

                        // And if we got a valid position. 
                        if let Some(ref position) = *position.borrow() { 

                            // Get the list of all valid ModelIndex for the current filter and the current position.
                            let matches = matches.borrow();
                            let matches_original_from_filter = matches.iter().filter(|x| x.1.is_some()).map(|x| x.0.get()).collect::<Vec<&ModelIndex>>();
                            let model_index = matches_original_from_filter[*position];

                            // If the position is still valid (not required, but just in case)...
                            if model_index.is_valid() {
                                unsafe { item = model.as_mut().unwrap().item_from_index(model_index); }
                                let text = unsafe { item.as_mut().unwrap().text().to_std_string() };
                                replaced_text = text.replace(&text_source, &text_replace);

                                // We need to do an extra check to ensure the new text can be in the field. Return in bools, as we don't support those columns.
                                match table_definition.fields[model_index.column() as usize].field_type {
                                    FieldType::Boolean => return,
                                    FieldType::Float => if replaced_text.parse::<f32>().is_err() { return show_dialog(app_ui.window, false, ErrorKind::DBTableReplaceInvalidData) }
                                    FieldType::Integer => if replaced_text.parse::<i32>().is_err() { return show_dialog(app_ui.window, false, ErrorKind::DBTableReplaceInvalidData) }
                                    FieldType::LongInteger => if replaced_text.parse::<i64>().is_err() { return show_dialog(app_ui.window, false, ErrorKind::DBTableReplaceInvalidData) }
                                    _ =>  {}
                                }
                            } else { return }
                        } else { return }

                        match table_definition.fields[unsafe { item.as_mut().unwrap().column() as usize }].field_type {
                            FieldType::Float => unsafe { item.as_mut().unwrap().set_data((&Variant::new2(replaced_text.parse::<f32>().unwrap()), 2)); }
                            FieldType::Integer => unsafe { item.as_mut().unwrap().set_data((&Variant::new0(replaced_text.parse::<i32>().unwrap()), 2)); }
                            FieldType::LongInteger => unsafe { item.as_mut().unwrap().set_data((&Variant::new2(replaced_text.parse::<i64>().unwrap()), 2)); }
                            _ => unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str(&replaced_text)); }
                        }

                        // If we still have matches, select the next match, if any, or the first one, if any.
                        if let Some(pos) = *position.borrow() {
                            let matches = matches.borrow();
                            let matches_in_filter = matches.iter().filter(|x| x.1.is_some()).map(|x| x.1.as_ref().unwrap().get()).collect::<Vec<&ModelIndex>>();
                           
                            let selection_model = unsafe { table_view.as_mut().unwrap().selection_model() };
                            unsafe { selection_model.as_mut().unwrap().select((
                                matches_in_filter[pos],
                                Flags::from_enum(SelectionFlag::ClearAndSelect)
                            )); }
                        }
                    }
                }
            )),

            // Slot for the "Replace All" button. This triggers the main save function, so we can let that one update the search stuff.
            slot_replace_all: SlotNoArgs::new(clone!(
                table_definition,
                packed_file_path,
                table_state_data,
                app_ui,
                matches => move || {
                    
                    // Get the texts and only proceed if the source is not empty.
                    let text_source = unsafe { search_line_edit.as_mut().unwrap().text().to_std_string() };
                    let text_replace = unsafe { replace_line_edit.as_mut().unwrap().text().to_std_string() };
                    if text_source == text_replace { return }
                    if !text_source.is_empty() {

                        // This is done like that because problems with borrowing matches and position. We cannot set the new text while
                        // matches is borrowed, so we have to catch that into his own scope.
                        let mut positions_and_texts: Vec<((i32, i32), String)> = vec![];
                        { 
                            // Get the list of all valid ModelIndex for the current filter and the current position.
                            let matches = matches.borrow();
                            let matches_original_from_filter = matches.iter().filter(|x| x.1.is_some()).map(|x| x.0.get()).collect::<Vec<&ModelIndex>>();
                            for model_index in &matches_original_from_filter {
                             
                                // If the position is still valid (not required, but just in case)...
                                if model_index.is_valid() {

                                    // We need to do an extra check to ensure every new field is valid. If one fails, return.
                                    let text = unsafe { model.as_ref().unwrap().item_from_index(model_index).as_mut().unwrap().text().to_std_string() };
                                    let replaced_text = text.replace(&text_source, &text_replace);
                                    match table_definition.fields[model_index.column() as usize].field_type {
                                        FieldType::Boolean => return,
                                        FieldType::Float => if replaced_text.parse::<f32>().is_err() { return show_dialog(app_ui.window, false, ErrorKind::DBTableReplaceInvalidData) }
                                        FieldType::Integer => if replaced_text.parse::<i32>().is_err() { return show_dialog(app_ui.window, false, ErrorKind::DBTableReplaceInvalidData) }
                                        FieldType::LongInteger => if replaced_text.parse::<i64>().is_err() { return show_dialog(app_ui.window, false, ErrorKind::DBTableReplaceInvalidData) }
                                        _ =>  {}
                                    }
                                } else { return }
                            }
                            let regex = Regex::new(&format!("(?i){}", text_source)).unwrap();
                            for model_index in &matches_original_from_filter {
                             
                                // If the position is still valid (not required, but just in case)...
                                if model_index.is_valid() {
                                    let item = unsafe { model.as_mut().unwrap().item_from_index(model_index) };
                                    let text = unsafe { item.as_mut().unwrap().text().to_std_string() };
                                    positions_and_texts.push(((model_index.row(), model_index.column()), regex.replace_all(&text, &*text_replace).to_string()));
                                } else { return }
                            }
                        }

                        // For each position, get his item and change his text.
                        for (index, data) in positions_and_texts.iter().enumerate() {
                            let item = unsafe { model.as_mut().unwrap().item(((data.0).0, (data.0).1)) };

                            match table_definition.fields[unsafe { item.as_mut().unwrap().column() as usize }].field_type {
                                FieldType::Float => unsafe { item.as_mut().unwrap().set_data((&Variant::new2(data.1.parse::<f32>().unwrap()), 2)); }
                                FieldType::Integer => unsafe { item.as_mut().unwrap().set_data((&Variant::new0(data.1.parse::<i32>().unwrap()), 2)); }
                                FieldType::LongInteger => unsafe { item.as_mut().unwrap().set_data((&Variant::new2(data.1.parse::<i64>().unwrap()), 2)); }
                                _ => unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str(&data.1)); }
                            }

                            // If we finished replacing, fix the undo history to have all the previous changed merged into one.
                            if index == positions_and_texts.len() - 1 {

                                {
                                    let mut table_state_data = table_state_data.borrow_mut();
                                    let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                                    let len = table_state_data.undo_history.len();
                                    let mut edits_data = vec![];
                                    
                                    {
                                        let mut edits = table_state_data.undo_history.drain((len - positions_and_texts.len())..);
                                        for edit in &mut edits { if let TableOperations::Editing(mut edit) = edit { edits_data.append(&mut edit); }}
                                    }

                                    table_state_data.undo_history.push(TableOperations::Editing(edits_data));
                                    table_state_data.redo_history.clear();
                                    update_undo_model(model, table_state_data.undo_model); 
                                }

                                unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                            }
                        }
                    }
                }
            )),
        };

        // Actions for the TableView...
        unsafe { (table_view as *mut Widget).as_ref().unwrap().signals().custom_context_menu_requested().connect(&slots.slot_context_menu); }
        unsafe { (table_view_frozen as *mut Widget).as_ref().unwrap().signals().custom_context_menu_requested().connect(&slots.slot_context_menu); }
        unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().signals().section_moved().connect(&slots.slot_column_moved); }
        unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().signals().sort_indicator_changed().connect(&slots.slot_sort_order_column_changed); }
        //unsafe { table_view_frozen.as_mut().unwrap().horizontal_header().as_mut().unwrap().signals().sort_indicator_changed().connect(&slots.slot_sort_order_column_changed); }
        unsafe { model.as_mut().unwrap().signals().data_changed().connect(&slots.save_changes); }
        unsafe { model.as_mut().unwrap().signals().item_changed().connect(&slots.slot_item_changed); }
        unsafe { context_menu_add.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_add); }
        unsafe { context_menu_insert.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_insert); }
        unsafe { context_menu_delete.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_delete); }
        unsafe { context_menu_apply_maths_to_selection.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_apply_maths_to_selection); }
        unsafe { context_menu_rewrite_selection.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_rewrite_selection); }
        unsafe { context_menu_clone.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_clone); }
        unsafe { context_menu_clone_and_append.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_clone_and_append); }
        unsafe { context_menu_copy.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_copy); }
        unsafe { context_menu_copy_as_lua_table.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_copy_as_lua_table); }
        unsafe { context_menu_paste.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_paste); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_paste_as_new_lines); }
        unsafe { context_menu_paste_to_fill_selection.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_paste_to_fill_selection); }
        unsafe { context_menu_selection_invert.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_selection_invert); }
        unsafe { context_menu_sidebar.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_sidebar); }
        unsafe { context_menu_search.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_search); }
        unsafe { context_menu_import.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_import); }
        unsafe { context_menu_export.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_export); }

        unsafe { smart_delete.as_mut().unwrap().signals().triggered().connect(&slots.slot_smart_delete); }
        unsafe { context_menu_undo.as_mut().unwrap().signals().triggered().connect(&slots.slot_undo); }
        unsafe { context_menu_redo.as_mut().unwrap().signals().triggered().connect(&slots.slot_redo); }
        unsafe { undo_redo_enabler.as_mut().unwrap().signals().triggered().connect(&slots.slot_undo_redo_enabler); }

        unsafe { update_search_stuff.as_mut().unwrap().signals().triggered().connect(&slots.slot_update_search_stuff); }
        unsafe { search_button.as_mut().unwrap().signals().released().connect(&slots.slot_search); }
        unsafe { prev_match_button.as_mut().unwrap().signals().released().connect(&slots.slot_prev_match); }
        unsafe { next_match_button.as_mut().unwrap().signals().released().connect(&slots.slot_next_match); }
        unsafe { close_button.as_mut().unwrap().signals().released().connect(&slots.slot_close_search); }
        unsafe { replace_current_button.as_mut().unwrap().signals().released().connect(&slots.slot_replace_current); }
        unsafe { replace_all_button.as_mut().unwrap().signals().released().connect(&slots.slot_replace_all); }

        // Trigger the filter whenever the "filtered" text changes, the "filtered" column changes or the "Case Sensitive" button changes.
        unsafe { row_filter_line_edit.as_mut().unwrap().signals().text_changed().connect(&slots.slot_row_filter_change_text); }
        unsafe { row_filter_column_selector.as_mut().unwrap().signals().current_index_changed_c_int().connect(&slots.slot_row_filter_change_column); }
        unsafe { row_filter_case_sensitive_button.as_mut().unwrap().signals().toggled().connect(&slots.slot_row_filter_change_case_sensitive); }

        // Initial states for the Contextual Menu Actions.
        unsafe {
            context_menu_add.as_mut().unwrap().set_enabled(true);
            context_menu_insert.as_mut().unwrap().set_enabled(true);
            context_menu_delete.as_mut().unwrap().set_enabled(false);
            context_menu_apply_maths_to_selection.as_mut().unwrap().set_enabled(false);
            context_menu_rewrite_selection.as_mut().unwrap().set_enabled(false);
            context_menu_clone.as_mut().unwrap().set_enabled(false);
            context_menu_clone_and_append.as_mut().unwrap().set_enabled(false);
            context_menu_copy.as_mut().unwrap().set_enabled(false);
            context_menu_copy_as_lua_table.as_mut().unwrap().set_enabled(true);
            context_menu_paste.as_mut().unwrap().set_enabled(true);
            context_menu_paste_as_new_lines.as_mut().unwrap().set_enabled(true);
            context_menu_paste_to_fill_selection.as_mut().unwrap().set_enabled(true);
            context_menu_selection_invert.as_mut().unwrap().set_enabled(true);
            context_menu_import.as_mut().unwrap().set_enabled(true);
            context_menu_export.as_mut().unwrap().set_enabled(true);
            undo_redo_enabler.as_mut().unwrap().trigger();
        }

        // Trigger the "Enable/Disable" slot every time we change the selection in the TreeView.
        unsafe { table_view.as_mut().unwrap().selection_model().as_ref().unwrap().signals().selection_changed().connect(&slots.slot_context_menu_enabler); }

        // If we got an entry for this PackedFile in the state's history, use it.
        if TABLE_STATES_UI.lock().unwrap().get(&*packed_file_path.borrow()).is_some() {
            let state_data;
            {
                let ts = TABLE_STATES_UI.lock().unwrap().clone();
                state_data = ts.get(&*packed_file_path.borrow()).clone().unwrap().clone();
            }
            // Ensure that the selected column actually exists in the table.
            let column = if state_data.filter_state.column < table_definition.fields.len() as i32 { state_data.filter_state.column } else { 0 };

            // Block the signals during this, so we don't trigger a borrow error.
            let mut blocker1 = unsafe { SignalBlocker::new(row_filter_line_edit.as_mut().unwrap().static_cast_mut() as &mut Object) };
            let mut blocker2 = unsafe { SignalBlocker::new(row_filter_column_selector.as_mut().unwrap().static_cast_mut() as &mut Object) };
            let mut blocker3 = unsafe { SignalBlocker::new(row_filter_case_sensitive_button.as_mut().unwrap().static_cast_mut() as &mut Object) };
            unsafe { row_filter_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&state_data.filter_state.text)); }
            unsafe { row_filter_column_selector.as_mut().unwrap().set_current_index(column); }
            unsafe { row_filter_case_sensitive_button.as_mut().unwrap().set_checked(state_data.filter_state.is_case_sensitive); }
            blocker1.unblock();
            blocker2.unblock();
            blocker3.unblock();

            // Ensure that the selected column actually exists in the table.
            let column = if state_data.search_state.column < table_definition.fields.len() as i32 { state_data.search_state.column } else { 0 };

            // Same with everything inside the search widget.
            let mut blocker1 = unsafe { SignalBlocker::new(search_line_edit.as_mut().unwrap().static_cast_mut() as &mut Object) };
            let mut blocker2 = unsafe { SignalBlocker::new(replace_line_edit.as_mut().unwrap().static_cast_mut() as &mut Object) };
            let mut blocker3 = unsafe { SignalBlocker::new(column_selector.as_mut().unwrap().static_cast_mut() as &mut Object) };
            let mut blocker4 = unsafe { SignalBlocker::new(case_sensitive_button.as_mut().unwrap().static_cast_mut() as &mut Object) };
            unsafe { search_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&state_data.search_state.search_text)); }
            unsafe { replace_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&state_data.search_state.replace_text)); }
            unsafe { column_selector.as_mut().unwrap().set_current_index(column); }
            unsafe { case_sensitive_button.as_mut().unwrap().set_checked(state_data.search_state.is_case_sensitive); }
            blocker1.unblock();
            blocker2.unblock();
            blocker3.unblock();
            blocker4.unblock();

            // Same with the columns, if we opted to keep their state.
            let mut blocker1 = unsafe { SignalBlocker::new(table_view.as_mut().unwrap().static_cast_mut() as &mut Object) };
            let mut blocker2 = unsafe { SignalBlocker::new(table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().static_cast_mut() as &mut Object) };
            
            // Depending on the current settings, load the current state of the table or not.
            if SETTINGS.lock().unwrap().settings_bool["remember_column_sorting"] {
                let sort_order = match state_data.columns_state.sorting_column.1 { 
                    1 => (state_data.columns_state.sorting_column.0, SortOrder::Ascending),
                    2 => (state_data.columns_state.sorting_column.0, SortOrder::Descending),
                    _ => (-1, SortOrder::Ascending),
                };
                unsafe { table_view.as_mut().unwrap().sort_by_column(sort_order); }
            }

            if SETTINGS.lock().unwrap().settings_bool["remember_column_visual_order"] {
                for change in &state_data.columns_state.visual_history {
                    match change {
                        VisualHistory::ColumnFrozen(_, logical_index, _) => {
                            unsafe { actions_freeze_unfreeze_column.borrow()[*logical_index as usize].as_mut().unwrap().toggle(); }
                            if let Some(state) = TABLE_STATES_UI.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
                                state.columns_state.visual_history.pop();
                            }
                        }
                        VisualHistory::ColumnMoved(old, new) => {
                            unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().move_section(*old, *new); }
                            unsafe { table_view_frozen.as_mut().unwrap().horizontal_header().as_mut().unwrap().move_section(*old, *new); }
                        }
                        VisualHistory::ColumnHidden(_, logical_index) => {
                            unsafe { actions_hide_show_column.borrow()[*logical_index as usize].as_mut().unwrap().toggle(); }
                            if let Some(state) = TABLE_STATES_UI.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
                                state.columns_state.visual_history.pop();
                            }
                        }
                    }
                }
            }
            
            blocker1.unblock();
            blocker2.unblock();
        }

        // Otherwise, we create a basic state.
        else { TABLE_STATES_UI.lock().unwrap().insert(packed_file_path.borrow().to_vec(), TableStateUI::new_empty()); }

        // Retrigger the filter, so the table get's updated properly.
        unsafe { row_filter_case_sensitive_button.as_mut().unwrap().set_checked(!row_filter_case_sensitive_button.as_mut().unwrap().is_checked()); }
        unsafe { row_filter_case_sensitive_button.as_mut().unwrap().set_checked(!row_filter_case_sensitive_button.as_mut().unwrap().is_checked()); }

        // Return the slots to keep them as hostages.
        Ok(slots)
    }

    /// This function loads the data from a LocData into a TableView.
    pub fn load_data_to_table_view(
        table_view: *mut TableView,
        model: *mut StandardItemModel,
        data: &TableType,
        table_definition: &Definition,
        dependency_data: &BTreeMap<i32, Vec<String>>,
    ) {
        // First, we delete all the data from the `ListStore`. Just in case there is something there.
        // This wipes out header information, so remember to run "build_columns" after this.
        unsafe { model.as_mut().unwrap().clear(); }

        // Set the right data, depending on the table type you get.
        let data = match data {
            TableType::DependencyManager(data) => &data,
            TableType::DB(data) => &data.entries,
            TableType::LOC(data) => &data.entries,
        };

        for entry in data {
            let mut qlist = ListStandardItemMutPtr::new(());
            for (index, field) in entry.iter().enumerate() {

                // Create a new Item.
                let item = match *field {

                    // This one needs a couple of changes before turning it into an item in the table.
                    DecodedData::Boolean(ref data) => {
                        let mut item = StandardItem::new(());
                        item.set_editable(false);
                        item.set_checkable(true);
                        item.set_check_state(if *data { CheckState::Checked } else { CheckState::Unchecked });
                        item
                    }

                    // Floats need to be tweaked to fix trailing zeroes and precission issues, like turning 0.5000004 into 0.5.
                    // Also, they should be limited to 3 decimals.
                    DecodedData::Float(ref data) => {
                        let data = {
                            let data_str = format!("{}", data);
                            if let Some(position) = data_str.find('.') {
                                let decimals = &data_str[position..].len();
                                if *decimals > 3 { format!("{:.3}", data).parse::<f32>().unwrap() }
                                else { *data }
                            }
                            else { *data }
                        };

                        let mut item = StandardItem::new(());
                        item.set_data((&Variant::new2(data), 2));
                        item
                    },
                    DecodedData::Integer(ref data) => {
                        let mut item = StandardItem::new(());
                        item.set_data((&Variant::new0(*data), 2));
                        item
                    },
                    DecodedData::LongInteger(ref data) => {
                        let mut item = StandardItem::new(());
                        item.set_data((&Variant::new2(*data), 2));
                        item
                    },
                    // All these are Strings, so it can be together,
                    DecodedData::StringU8(ref data) |
                    DecodedData::StringU16(ref data) |
                    DecodedData::OptionalStringU8(ref data) |
                    DecodedData::OptionalStringU16(ref data) => StandardItem::new(&QString::from_std_str(data)),
                };

                // If we have the dependency stuff enabled, check if it's a valid reference.
                if SETTINGS.lock().unwrap().settings_bool["use_dependency_checker"] && table_definition.fields[index].field_is_reference.is_some() {
                    Self::check_references(dependency_data, index as i32, item.as_mut_ptr());
                }

                unsafe { qlist.append_unsafe(&item.into_raw()); }
            }
            unsafe { model.as_mut().unwrap().append_row(&qlist); }
        }

        // If the table it's empty, we add an empty row and delete it, so the "columns" get created.
        if data.is_empty() {
            let mut qlist = ListStandardItemMutPtr::new(());
            for field in &table_definition.fields {
                let item = match field.field_type {
                    FieldType::Boolean => {
                        let mut item = StandardItem::new(());
                        item.set_editable(false);
                        item.set_checkable(true);
                        item.set_check_state(CheckState::Checked);
                        item
                    }
                    FieldType::Float => {
                        let mut item = StandardItem::new(());
                        item.set_data((&Variant::new2(0.0f32), 2));
                        item
                    },
                    FieldType::Integer => {
                        let mut item = StandardItem::new(());
                        item.set_data((&Variant::new0(0i32), 2));
                        item
                    },
                    FieldType::LongInteger => {
                        let mut item = StandardItem::new(());
                        item.set_data((&Variant::new2(0i64), 2));
                        item
                    },
                    FieldType::StringU8 |
                    FieldType::StringU16 |
                    FieldType::OptionalStringU8 |
                    FieldType::OptionalStringU16 => StandardItem::new(&QString::from_std_str("")),
                };
                unsafe { qlist.append_unsafe(&item.into_raw()); }
            }
            unsafe { model.as_mut().unwrap().append_row(&qlist); }
            unsafe { model.as_mut().unwrap().remove_rows((0, 1)); }
        }

        // Here we assing the ItemDelegates, so each type has his own widget with validation included.
        // LongInteger uses normal string controls due to QSpinBox being limited to i32.
        // The rest don't need any kind of validation. For now.
        for (column, field) in table_definition.fields.iter().enumerate() {
            match field.field_type {
                FieldType::Boolean => {},
                FieldType::Float => unsafe { qt_custom_stuff::new_doublespinbox_item_delegate(table_view as *mut Object, column as i32) },
                FieldType::Integer => unsafe { qt_custom_stuff::new_spinbox_item_delegate(table_view as *mut Object, column as i32, 32) },
                FieldType::LongInteger => unsafe { qt_custom_stuff::new_spinbox_item_delegate(table_view as *mut Object, column as i32, 64) },
                FieldType::StringU8 => {},
                FieldType::StringU16 => {},
                FieldType::OptionalStringU8 => {},
                FieldType::OptionalStringU16 => {},
            }
        }

        // We build the combos lists here, so it get's rebuilt if we import a TSV and clear the table.
        if !SETTINGS.lock().unwrap().settings_bool["disable_combos_on_tables"] {
            for (column, data) in dependency_data {
                let mut list = StringList::new(());
                data.iter().for_each(|x| list.append(&QString::from_std_str(x)));
                let list: *mut StringList = &mut list;
                unsafe { qt_custom_stuff::new_combobox_item_delegate(table_view as *mut Object, *column, list as *const StringList, true)};
            }
        }
    }

    /// This function returns a DBData with all the stuff in the table. The data is filtered in the UI BEFORE inserting it
    /// into the table, so this should be safe. Should.
    pub fn return_data_from_table_view(
        table_type: &mut TableType,
        definition: &Definition,
        model: *mut StandardItemModel,
    ) {

        // Remove every entry from the DB, then add each one again from the model.
        // The model already validates the data BEFORE accepting it, so all the data here should be valid.
        let packed_file_data = match table_type {
            TableType::DependencyManager(data) => data,
            TableType::DB(data) => &mut data.entries,
            TableType::LOC(data) => &mut data.entries,
        };

        packed_file_data.clear();
        for row in 0..unsafe { model.as_mut().unwrap().row_count(()) } {
            let mut new_row: Vec<DecodedData> = vec![];
            for (column, field) in definition.fields.iter().enumerate() {

                // Create a new Item.
                let item = unsafe {
                    match field.field_type {

                        // This one needs a couple of changes before turning it into an item in the table.
                        FieldType::Boolean => DecodedData::Boolean(if model.as_mut().unwrap().item((row as i32, column as i32)).as_mut().unwrap().check_state() == CheckState::Checked { true } else { false }),

                        // Numbers need parsing, and this can fail.
                        FieldType::Float => DecodedData::Float(model.as_mut().unwrap().item((row as i32, column as i32)).as_mut().unwrap().data(2).to_float()),
                        FieldType::Integer => DecodedData::Integer(model.as_mut().unwrap().item((row as i32, column as i32)).as_mut().unwrap().data(2).to_int()),
                        FieldType::LongInteger => DecodedData::LongInteger(model.as_mut().unwrap().item((row as i32, column as i32)).as_mut().unwrap().data(2).to_long_long()),

                        // All these are just normal Strings.
                        FieldType::StringU8 => DecodedData::StringU8(QString::to_std_string(&model.as_mut().unwrap().item((row as i32, column as i32)).as_mut().unwrap().text())),
                        FieldType::StringU16 => DecodedData::StringU16(QString::to_std_string(&model.as_mut().unwrap().item((row as i32, column as i32)).as_mut().unwrap().text())),
                        FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(QString::to_std_string(&model.as_mut().unwrap().item((row as i32, column as i32)).as_mut().unwrap().text())),
                        FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(QString::to_std_string(&model.as_mut().unwrap().item((row as i32, column as i32)).as_mut().unwrap().text())),
                    }
                };
                new_row.push(item);
            }
            packed_file_data.push(new_row);
        }
    }

    /// Function to save the data from the current StandardItemModel to the PackFile.
    pub fn save_to_packed_file(
        sender_qt: &Sender<Commands>,
        sender_qt_data: &Sender<Data>,
        receiver_qt: &Rc<RefCell<Receiver<Data>>>,
        app_ui: &AppUI,
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        model: *mut StandardItemModel,
        global_search_explicit_paths: &Rc<RefCell<Vec<Vec<String>>>>,
        update_global_search_stuff: *mut Action,
        definition: &Definition,
        mut table_type: &mut TableType,
    ) {

        // Update the DB with the data in the table, or report error if it fails.
        Self::return_data_from_table_view(&mut table_type, definition, model);
        
        // Depending on the table type, we save it one way or another.
        match table_type {
            TableType::DependencyManager(pack_file_list) => {
                let pack_file_list = pack_file_list.iter().map(|x| if let DecodedData::StringU8(data) = &x[0] { data.to_owned() } else { unimplemented!() }).collect();
                sender_qt.send(Commands::SetPackFilesList).unwrap();
                sender_qt_data.send(Data::VecString(pack_file_list)).unwrap();
            },

            TableType::DB(packed_file) => {
                sender_qt.send(Commands::EncodePackedFileDB).unwrap();
                sender_qt_data.send(Data::DBVecString((packed_file.clone(), packed_file_path.borrow().to_vec()))).unwrap();
            },

            TableType::LOC(packed_file) => {
                sender_qt.send(Commands::EncodePackedFileLoc).unwrap();
                sender_qt_data.send(Data::LocVecString((packed_file.clone(), packed_file_path.borrow().to_vec()))).unwrap();
            }
        }

        let tree_path_type = match table_type {
            TableType::DependencyManager(_) => TreePathType::PackFile,
            TableType::DB(_) | TableType::LOC(_) => TreePathType::File(packed_file_path.borrow().to_vec()), 
        };

        update_treeview(
            &sender_qt,
            &sender_qt_data,
            &receiver_qt,
            &app_ui,
            app_ui.folder_tree_view,
            Some(app_ui.folder_tree_filter),
            app_ui.folder_tree_model,
            TreeViewOperation::Modify(vec![tree_path_type]),
        );

        // Update the global search stuff, if needed.
        global_search_explicit_paths.borrow_mut().push(packed_file_path.borrow().to_vec());
        unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }
    }

    /// Function to undo/redo an operation in the table.
    /// - history_source: the history used to "go back".
    /// - history_opposite: the history used to "go back" the action we are doing.
    /// The rest is just usual stuff used to save tables.
    pub fn undo_redo(
        app_ui: &AppUI,
        dependency_data: &Rc<BTreeMap<i32, Vec<String>>>,
        sender_qt: &Sender<Commands>,
        sender_qt_data: &Sender<Data>,
        receiver_qt: &Rc<RefCell<Receiver<Data>>>,
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        table_view: *mut TableView,
        table_view_frozen: *mut TableView,
        model: *mut StandardItemModel,
        filter_model: *mut SortFilterProxyModel,
        history_source: &mut Vec<TableOperations>, 
        history_opposite: &mut Vec<TableOperations>,
        global_search_explicit_paths: &Rc<RefCell<Vec<Vec<String>>>>,
        update_global_search_stuff: *mut Action,
        undo_lock: &Rc<RefCell<bool>>,
        save_lock: &Rc<RefCell<bool>>,
        table_definition: &Definition,
        table_type: &Rc<RefCell<TableType>>,
        enable_header_popups: Option<String>,
    ) {

        // Get the last operation in the Undo History, or return if there is none.
        let operation = if let Some(operation) = history_source.pop() { operation } else { return };
        match operation {
            TableOperations::Editing(editions) => {

                // Prepare the redo operation, then do the rest.
                let mut redo_editions = vec![];
                unsafe { editions.iter().for_each(|x| redo_editions.push((((x.0).0, (x.0).1), (&*model.as_mut().unwrap().item(((x.0).0, (x.0).1))).clone()))); }
                history_opposite.push(TableOperations::Editing(redo_editions));
    
                *undo_lock.borrow_mut() = true;
                *save_lock.borrow_mut() = true;
                for (index, edit) in editions.iter().enumerate() {
                    let row = (edit.0).0;
                    let column = (edit.0).1;
                    let item = edit.1;
                    unsafe { model.as_mut().unwrap().set_item((row, column, item.clone())); }
                    
                    // If we are going to process the last one, unlock the save.
                    if index == editions.len() - 1 { 
                        unsafe { model.as_mut().unwrap().item((row, column)).as_mut().unwrap().set_data((&Variant::new0(1i32), 16)); }
                        *save_lock.borrow_mut() = false;
                        unsafe { model.as_mut().unwrap().item((row, column)).as_mut().unwrap().set_data((&Variant::new0(()), 16)); }
                    }
                }
                *undo_lock.borrow_mut() = false;
    
                // Select all the edited items.
                let selection_model = unsafe { table_view.as_mut().unwrap().selection_model() };
                unsafe { selection_model.as_mut().unwrap().clear(); }
                for ((row, column),_) in &editions {
                    let model_index_filtered = unsafe { filter_model.as_ref().unwrap().map_from_source(&model.as_mut().unwrap().index((*row, *column))) };
                    if model_index_filtered.is_valid() {
                        unsafe { selection_model.as_mut().unwrap().select((
                            &model_index_filtered,
                            Flags::from_enum(SelectionFlag::Select)
                        )); }
                    }
                }
            }

            // This action is special and we have to manually trigger a save for it.
            // This actions if for undoing "add rows" actions. It deletes the stored rows.
            // NOTE: the rows list must ALWAYS be in 9->1 order. Otherwise this breaks.
            TableOperations::AddRows(rows) => {

                // Split the row list in consecutive rows, get their data, and remove them in batches.
                let mut rows_splitted = vec![];
                let mut current_row_pack = vec![];
                let mut current_row_index = -2;
                for (index, row) in rows.iter().enumerate() {

                    let mut items = vec![];                        
                    for column in 0..unsafe { model.as_mut().unwrap().column_count(()) } {
                        let item = unsafe { &*model.as_mut().unwrap().item((*row, column)) };
                        items.push(item.clone());
                    }

                    if (*row == current_row_index - 1) || index == 0 { 
                        current_row_pack.push((*row, items)); 
                        current_row_index = *row;
                    }
                    else {
                        current_row_pack.reverse();
                        rows_splitted.push(current_row_pack.to_vec());
                        current_row_pack.clear();
                        current_row_pack.push((*row, items)); 
                        current_row_index = *row;
                    }
                }
                current_row_pack.reverse();
                rows_splitted.push(current_row_pack);

                for row_pack in rows_splitted.iter() {
                    unsafe { model.as_mut().unwrap().remove_rows((row_pack[0].0, row_pack.len() as i32)); }
                }

                rows_splitted.reverse();
                history_opposite.push(TableOperations::RemoveRows(rows_splitted));

                Self::save_to_packed_file(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &app_ui,
                    &packed_file_path,
                    model,
                    &global_search_explicit_paths,
                    update_global_search_stuff,
                    table_definition,
                    &mut table_type.borrow_mut(),
                );
            }

            // NOTE: the rows list must ALWAYS be in 1->9 order. Otherwise this breaks.
            TableOperations::RemoveRows(rows) => {

                // First, we re-insert the pack of empty rows. Then, we put the data into them. And repeat with every Pack.
                for row_pack in &rows {
                    for (row, items) in row_pack {
                        let mut qlist = ListStandardItemMutPtr::new(());
                        unsafe { items.iter().for_each(|x| qlist.append_unsafe(x)); }
                        unsafe { model.as_mut().unwrap().insert_row((*row, &qlist)); }
                    }
                }

                // Create the "redo" action for this one.
                let mut rows_to_add = vec![];
                rows.to_vec().iter_mut().map(|x| x.iter_mut().map(|y| y.0).collect::<Vec<i32>>()).for_each(|mut x| rows_to_add.append(&mut x));
                rows_to_add.reverse();
                history_opposite.push(TableOperations::AddRows(rows_to_add));
                
                // Select all the re-inserted rows that are in the filter. We need to block signals here because the bigger this gets, the slower it gets. And it gets very slow.
                let selection_model = unsafe { table_view.as_mut().unwrap().selection_model() };
                unsafe { selection_model.as_mut().unwrap().clear(); }
                for row_pack in &rows {
                    let initial_model_index_filtered = unsafe { filter_model.as_ref().unwrap().map_from_source(&model.as_mut().unwrap().index((row_pack[0].0, 0))) };
                    let final_model_index_filtered = unsafe { filter_model.as_ref().unwrap().map_from_source(&model.as_mut().unwrap().index((row_pack.last().unwrap().0 as i32, 0))) };
                    if initial_model_index_filtered.is_valid() && final_model_index_filtered.is_valid() {
                        let selection = ItemSelection::new((&initial_model_index_filtered, &final_model_index_filtered));
                        unsafe { selection_model.as_mut().unwrap().select((&selection, Flags::from_enum(SelectionFlag::Select) | Flags::from_enum(SelectionFlag::Rows))); }
                    }
                }

                // Trick to tell the model to update everything.
                *undo_lock.borrow_mut() = true;
                unsafe { model.as_mut().unwrap().item((0, 0)).as_mut().unwrap().set_data((&Variant::new0(()), 16)); }
                *undo_lock.borrow_mut() = false;
            }

            // "rows" has to come in the same format than in RemoveRows.
            TableOperations::SmartDelete((edits, rows)) => {

                // First, we re-insert each pack of rows.
                for row_pack in &rows {
                    for (row, items) in row_pack {
                        let mut qlist = ListStandardItemMutPtr::new(());
                        unsafe { items.iter().for_each(|x| qlist.append_unsafe(x)); }
                        unsafe { model.as_mut().unwrap().insert_row((*row, &qlist)); }
                    }
                }

                // Then, restore all the edits and keep their old state for the undo/redo action.
                *undo_lock.borrow_mut() = true;
                let edits_before = unsafe { edits.iter().map(|x| (((x.0).0, (x.0).1), (&*model.as_mut().unwrap().item(((x.0).0, (x.0).1))).clone())).collect::<Vec<((i32, i32), *mut StandardItem)>>() };
                unsafe { edits.iter().for_each(|x| model.as_mut().unwrap().set_item(((x.0).0, (x.0).1, x.1.clone()))); }
                *undo_lock.borrow_mut() = false;

                // Next, prepare the redo operation.
                let mut rows_to_add = vec![];
                rows.to_vec().iter_mut().map(|x| x.iter_mut().map(|y| y.0).collect::<Vec<i32>>()).for_each(|mut x| rows_to_add.append(&mut x));
                rows_to_add.reverse();
                history_opposite.push(TableOperations::RevertSmartDelete((edits_before, rows_to_add)));

                // Select all the edited items/restored rows.
                let selection_model = unsafe { table_view.as_mut().unwrap().selection_model() };
                unsafe { selection_model.as_mut().unwrap().clear(); }
                for row_pack in &rows {
                    let initial_model_index_filtered = unsafe { filter_model.as_ref().unwrap().map_from_source(&model.as_mut().unwrap().index((row_pack[0].0, 0))) };
                    let final_model_index_filtered = unsafe { filter_model.as_ref().unwrap().map_from_source(&model.as_mut().unwrap().index((row_pack.last().unwrap().0 as i32, 0))) };
                    if initial_model_index_filtered.is_valid() && final_model_index_filtered.is_valid() {
                        let selection = ItemSelection::new((&initial_model_index_filtered, &final_model_index_filtered));
                        unsafe { selection_model.as_mut().unwrap().select((&selection, Flags::from_enum(SelectionFlag::Select) | Flags::from_enum(SelectionFlag::Rows))); }
                    }
                }

                for edit in edits.iter() {
                    let model_index_filtered = unsafe { filter_model.as_ref().unwrap().map_from_source(&model.as_mut().unwrap().index(((edit.0).0, (edit.0).1))) };
                    if model_index_filtered.is_valid() {
                        unsafe { selection_model.as_mut().unwrap().select((
                            &model_index_filtered,
                            Flags::from_enum(SelectionFlag::Select)
                        )); }
                    }
                }

                // Trick to tell the model to update everything.
                *undo_lock.borrow_mut() = true;
                unsafe { model.as_mut().unwrap().item((0, 0)).as_mut().unwrap().set_data((&Variant::new0(()), 16)); }
                *undo_lock.borrow_mut() = false;
            }
                        
            // This action is special and we have to manually trigger a save for it.
            // "rows" has to come in the same format than in AddRows.
            TableOperations::RevertSmartDelete((edits, rows)) => {

                // First, redo all the "edits".
                *undo_lock.borrow_mut() = true;
                let edits_before = unsafe { edits.iter().map(|x| (((x.0).0, (x.0).1), (&*model.as_mut().unwrap().item(((x.0).0, (x.0).1))).clone())).collect::<Vec<((i32, i32), *mut StandardItem)>>() };
                unsafe { edits.iter().for_each(|x| model.as_mut().unwrap().set_item(((x.0).0, (x.0).1, x.1.clone()))); }
                *undo_lock.borrow_mut() = false;

                // Select all the edited items, if any, before removing rows. Otherwise, the selection will not match the editions.
                let selection_model = unsafe { table_view.as_mut().unwrap().selection_model() };
                unsafe { selection_model.as_mut().unwrap().clear(); }
                for edit in edits.iter() {
                    let model_index_filtered = unsafe { filter_model.as_ref().unwrap().map_from_source(&model.as_mut().unwrap().index(((edit.0).0, (edit.0).1))) };
                    if model_index_filtered.is_valid() {
                        unsafe { selection_model.as_mut().unwrap().select((
                            &model_index_filtered,
                            Flags::from_enum(SelectionFlag::Select)
                        )); }
                    }
                }

                // Then, remove the restored tables after undoing a "SmartDelete".
                // Same thing as with "AddRows": split the row list in consecutive rows, get their data, and remove them in batches.
                let mut rows_splitted = vec![];
                let mut current_row_pack = vec![];
                let mut current_row_index = -2;
                for (index, row) in rows.iter().enumerate() {

                    let mut items = vec![];                        
                    for column in 0..unsafe { model.as_mut().unwrap().column_count(()) } {
                        let item = unsafe { &*model.as_mut().unwrap().item((*row, column)) };
                        items.push(item.clone());
                    }

                    if (*row == current_row_index - 1) || index == 0 { 
                        current_row_pack.push((*row, items)); 
                        current_row_index = *row;
                    }
                    else {
                        current_row_pack.reverse();
                        rows_splitted.push(current_row_pack.to_vec());
                        current_row_pack.clear();
                        current_row_pack.push((*row, items)); 
                        current_row_index = *row;
                    }
                }
                current_row_pack.reverse();
                rows_splitted.push(current_row_pack);
                if rows_splitted[0].is_empty() { rows_splitted.clear(); }

                for row_pack in rows_splitted.iter() {
                    unsafe { model.as_mut().unwrap().remove_rows((row_pack[0].0, row_pack.len() as i32)); }
                }

                // Prepare the redo operation.
                rows_splitted.reverse();
                history_opposite.push(TableOperations::SmartDelete((edits_before, rows_splitted)));

                // Try to save the PackedFile to the main PackFile.
                Self::save_to_packed_file(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &app_ui,
                    &packed_file_path,
                    model,
                    &global_search_explicit_paths,
                    update_global_search_stuff,
                    table_definition,
                    &mut table_type.borrow_mut(),
                );
            }

            // This action is special and we have to manually trigger a save for it.
            TableOperations::ImportTSV(table_data) => {

                // Prepare the redo operation.
                {
                    let table_type = &mut *table_type.borrow_mut(); 
                    match table_type {
                        TableType::DependencyManager(data) => {
                            history_opposite.push(TableOperations::ImportTSV(data.to_vec()));
                            *data = table_data;
                        },
                        TableType::DB(data) => {
                            history_opposite.push(TableOperations::ImportTSV(data.entries.to_vec()));
                            data.entries = table_data;
                        },
                        TableType::LOC(data) => {
                            history_opposite.push(TableOperations::ImportTSV(data.entries.to_vec()));
                            data.entries = table_data;
                        },
                    }
                }

                Self::load_data_to_table_view(table_view, model, &table_type.borrow(), table_definition, &dependency_data);
                Self::build_columns(table_view, table_view_frozen, model, table_definition, enable_header_popups);

                // If we want to let the columns resize themselfs...
                if SETTINGS.lock().unwrap().settings_bool["adjust_columns_to_content"] {
                    unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
                }

                // Try to save the PackedFile to the main PackFile.
                Self::save_to_packed_file(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &app_ui,
                    &packed_file_path,
                    model,
                    &global_search_explicit_paths,
                    update_global_search_stuff,
                    table_definition,
                    &mut table_type.borrow_mut(),
                );
            }
            TableOperations::Carolina(operations) => {
                for operation in &operations {
                    history_source.push((*operation).clone());
                    Self::undo_redo(
                        &app_ui,
                        &dependency_data,
                        &sender_qt,
                        &sender_qt_data,
                        &receiver_qt,
                        &packed_file_path,
                        table_view,
                        table_view_frozen,
                        model,
                        filter_model,
                        history_source,
                        history_opposite,
                        &global_search_explicit_paths,
                        update_global_search_stuff,
                        &undo_lock,
                        &save_lock,
                        &table_definition,
                        &table_type,
                        enable_header_popups.clone()
                    );
                }
                let len = history_opposite.len();
                let mut edits = history_opposite.drain((len - operations.len())..).collect::<Vec<TableOperations>>();
                edits.reverse();
                history_opposite.push(TableOperations::Carolina(edits));
            }
        }
    }

    //----------------------------------------------------------------//
    // Helper Functions for DB/LOC PackedFiles.
    //----------------------------------------------------------------//

    /// This function checks if the PackFiles in the model are valid, and paints as red the invalid ones.
    fn check_dependency_packfile_errors( model: *mut StandardItemModel) {

        // For each row...
        let rows = unsafe { model.as_mut().unwrap().row_count(()) };
        for row in 0..rows {
            let item = unsafe { model.as_mut().unwrap().item((row as i32, 0)) };
            let packfile = unsafe { item.as_mut().unwrap().text().to_std_string() };

            // We paint it depending on if it's a valid PackFile or not.
            if !packfile.is_empty() && packfile.ends_with(".pack") && !packfile.contains(' ') { unsafe { item.as_mut().unwrap().set_foreground(&Brush::new(GlobalColor::Black)); } }
            else { unsafe { item.as_mut().unwrap().set_foreground(&Brush::new(GlobalColor::Red)); } }
        }  
    }

    /// This function "process" the column names of a table, so they look like they should.
    fn clean_column_names(field_name: &str) -> String {
        let mut new_name = String::new();
        let mut should_be_uppercase = false;

        for character in field_name.chars() {

            if new_name.is_empty() || should_be_uppercase {
                new_name.push_str(&character.to_uppercase().to_string());
                should_be_uppercase = false;
            }

            else if character == '_' {
                new_name.push(' ');
                should_be_uppercase = true;
            }

            else { new_name.push(character); }
        }

        new_name
    }

    /// This function is meant to be used to prepare and build the column headers, and the column-related stuff.
    /// His intended use is for just after we load/reload the data to the table.
    fn build_columns(
        table_view: *mut TableView,
        table_view_frozen: *mut TableView,
        model: *mut StandardItemModel,
        definition: &Definition,
        enable_header_popups: Option<String>,
    ) {
        // Create a list of "Key" columns.
        let mut keys = vec![];

        // For each column, clean their name and set their width and tooltip.
        for (index, field) in definition.fields.iter().enumerate() {

            let name = Self::clean_column_names(&field.field_name);
            let item = StandardItem::new(&QString::from_std_str(&name)).into_raw();
            unsafe { model.as_mut().unwrap().set_horizontal_header_item(index as i32, item) };

            // Depending on his type, set one width or another.
            match field.field_type {
                FieldType::Boolean => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 100); }
                FieldType::Float => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 140); }
                FieldType::Integer => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 140); }
                FieldType::LongInteger => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 140); }
                FieldType::StringU8 => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 350); }
                FieldType::StringU16 => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 350); }
                FieldType::OptionalStringU8 => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 350); }
                FieldType::OptionalStringU16 => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 350); }
            }

            // We only pass this for DB Tables. Loc files can skip this.
            if let Some(ref table_name) = enable_header_popups {   
        
                // Create the tooltip for the column. To get the reference data, we iterate through every table in the schema and check their references.
                let mut tooltip_text = String::new();
                if !field.field_description.is_empty() { tooltip_text.push_str(&format!("<p>{}</p>", field.field_description)); }
                if let Some(ref reference) = field.field_is_reference {
                    tooltip_text.push_str(&format!("<p>This column is a reference to:</p><p><i>\"{}/{}\"</i></p>", reference.0, reference.1));
                } else { 
                    let schema = SCHEMA.lock().unwrap().clone();
                    let mut referenced_columns = if let Some(schema) = schema {
                        let short_table_name = table_name.split_at(table_name.len() - 7).0;
                        let mut columns = vec![];
                        for table in schema.tables_definitions {
                            let mut found = false;
                            for version in table.versions {
                                for field_ref in version.fields {
                                    if let Some(ref_data) = field_ref.field_is_reference { 
                                        if ref_data.0 == short_table_name && ref_data.1 == field.field_name {
                                            found = true;
                                            columns.push((table.name.to_owned(), field_ref.field_name)); 
                                        }
                                    }
                                }
                                if found { break; }
                            }
                        }
                        columns
                    } else { vec![] };

                    referenced_columns.sort_unstable();
                    if !referenced_columns.is_empty() { 
                        tooltip_text.push_str("<p>Fields that reference this column:</p>");
                        for (index, reference) in referenced_columns.iter().enumerate() {
                            tooltip_text.push_str(&format!("<i>\"{}/{}\"</i><br>", reference.0, reference.1));

                            // There is a bug that causes tooltips to be displayed out of screen if they're too big. This fixes it.
                            if index == 50 { 
                                tooltip_text.push_str(&format!("<p>And many more. Exactly, {} more. Too many to show them here.</p>nnnn", referenced_columns.len() as isize - 50));
                                break ;
                            }
                        }

                        // Dirty trick to remove the last <br> from the tooltip, or the nnnn in case that text get used.
                        tooltip_text.pop();
                        tooltip_text.pop();
                        tooltip_text.pop();
                        tooltip_text.pop();
                    }
                }
                if !tooltip_text.is_empty() { unsafe { item.as_mut().unwrap().set_tool_tip(&QString::from_std_str(&tooltip_text)); }}
            }

            // If the field is key, add that column to the "Key" list, so we can move them at the begining later.
            if field.field_is_key { keys.push(index); }
        }

        // If we have any "Key" field, move it to the beginning.
        if !keys.is_empty() {
            for (position, column) in keys.iter().enumerate() {
                unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().move_section(*column as i32, position as i32); }
                unsafe { table_view_frozen.as_mut().unwrap().horizontal_header().as_mut().unwrap().move_section(*column as i32, position as i32); }
            }
        }
    }

    // Function to check if an specific field's data is in their references.
    fn check_references(
        dependency_data: &BTreeMap<i32, Vec<String>>,
        column: i32,
        item: *mut StandardItem,
    ) {
        // Check if it's a valid reference.
        if let Some(ref_data) = dependency_data.get(&column) {

            let text = unsafe { item.as_mut().unwrap().text().to_std_string() };
            if ref_data.contains(&text) { unsafe { item.as_mut().unwrap().set_foreground(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::White } else { GlobalColor::Black })); } }
            else if ref_data.is_empty() { unsafe { item.as_mut().unwrap().set_foreground(&Brush::new(GlobalColor::Blue)); } }
            else { unsafe { item.as_mut().unwrap().set_foreground(&Brush::new(GlobalColor::Red)); } }
        }
    }

    /// This function checks if the data in the clipboard is suitable for be pasted in all selected cells.
    fn check_clipboard_to_fill_selection(
        definition: &Definition,
        table_view: *mut TableView,
        model: *mut StandardItemModel,
        filter_model: *mut SortFilterProxyModel,
    ) -> bool {

        // Get the current selection.
        let clipboard = GuiApplication::clipboard();
        let text = unsafe { clipboard.as_mut().unwrap().text(()).to_std_string() };
        let indexes = unsafe { filter_model.as_mut().unwrap().map_selection_to_source(&table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };

        // If there is nothing selected, don't waste your time.
        if indexes.count(()) == 0 { return false }

        // For each selected index...
        for index in 0..indexes.count(()) {
            let model_index = indexes.at(index);
            if model_index.is_valid() {

                // Depending on the column, we try to encode the data in one format or another.
                let item = unsafe { model.as_mut().unwrap().item_from_index(&model_index) };
                let column = unsafe { item.as_mut().unwrap().index().column() };
                match definition.fields[column as usize].field_type {
                    FieldType::Boolean => if text.to_lowercase() != "true" && text.to_lowercase() != "false" && text != "1" && text != "0" { return false },
                    FieldType::Float => if text.parse::<f32>().is_err() { return false },
                    FieldType::Integer => if text.parse::<i32>().is_err() { return false },
                    FieldType::LongInteger => if text.parse::<i64>().is_err() { return false },

                    // All these are Strings, so we can skip their checks....
                    FieldType::StringU8 |
                    FieldType::StringU16 |
                    FieldType::OptionalStringU8 |
                    FieldType::OptionalStringU16 => {}
                }
            }
        }

        // If we reach this place, it means none of the cells was incorrect, so we can paste.
        true
    }

    /// This function checks if the data in the clipboard is suitable to be appended as rows at the end of the Table.
    fn check_clipboard_append_rows(
        table_view: *mut TableView,
        definition: &Definition
    ) -> bool {

        // Get the text from the clipboard.
        let clipboard = GuiApplication::clipboard();
        let mut text = unsafe { clipboard.as_mut().unwrap().text(()).to_std_string() };

        // If the text ends in \n, remove it. Excel things. We don't use newlines, so replace them with '\t'.
        if text.ends_with('\n') { text.pop(); }
        let text = text.replace('\n', "\t");
        let text = text.split('\t').collect::<Vec<&str>>();

        // Get the index for the column.
        let mut column = 0;
        for cell in text {

            // Depending on the column, we try to encode the data in one format or another.
            let column_logical_index = unsafe { table_view.as_ref().unwrap().horizontal_header().as_ref().unwrap().logical_index(column) };
            match definition.fields[column_logical_index as usize].field_type {
                FieldType::Boolean => if cell.to_lowercase() != "true" && cell.to_lowercase() != "false" && cell != "1" && cell != "0" { return false },
                FieldType::Float => if cell.parse::<f32>().is_err() { return false },
                FieldType::Integer => if cell.parse::<i32>().is_err() { return false },
                FieldType::LongInteger => if cell.parse::<i64>().is_err() { return false },

                // All these are Strings, so we can skip their checks....
                FieldType::StringU8 |
                FieldType::StringU16 |
                FieldType::OptionalStringU8 |
                FieldType::OptionalStringU16 => {}
            }

            // Reset or increase the column count, if needed.
            if column as usize == definition.fields.len() - 1 { column = 0; } else { column += 1; }
        }

        // If we reach this place, it means none of the cells was incorrect, so we can paste.
        true
    }

    /// Function to filter the table. If a value is not provided by a slot, we get it from the widget itself.
    fn filter_table(
        pattern: Option<QString>,
        column: Option<i32>,
        case_sensitive: Option<bool>,
        filter_model: *mut SortFilterProxyModel,
        filter_line_edit: *mut LineEdit,
        column_selector: *mut ComboBox,
        case_sensitive_button: *mut PushButton,
        update_search_stuff: *mut Action,
        packed_file_path: &Rc<RefCell<Vec<String>>>,
    ) {

        // Set the pattern to search.
        let mut pattern = if let Some(pattern) = pattern { RegExp::new(&pattern) }
        else { unsafe { RegExp::new(&filter_line_edit.as_mut().unwrap().text()) }};

        // Set the column selected.
        if let Some(column) = column { unsafe { filter_model.as_mut().unwrap().set_filter_key_column(column); }}
        else { unsafe { filter_model.as_mut().unwrap().set_filter_key_column(column_selector.as_mut().unwrap().current_index()); }}

        // Check if the filter should be "Case Sensitive".
        if let Some(case_sensitive) = case_sensitive { 
            if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::Sensitive); }
            else { pattern.set_case_sensitivity(CaseSensitivity::Insensitive); }
        }

        else {
            let case_sensitive = unsafe { case_sensitive_button.as_mut().unwrap().is_checked() };
            if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::Sensitive); }
            else { pattern.set_case_sensitivity(CaseSensitivity::Insensitive); }
        }

        // Filter whatever it's in that column by the text we got.
        unsafe { filter_model.as_mut().unwrap().set_filter_reg_exp(&pattern); }

        // Update the search stuff, if needed.
        unsafe { update_search_stuff.as_mut().unwrap().trigger(); }

        // Add the new filter data to the state history.
        if let Some(state) = TABLE_STATES_UI.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
            unsafe { state.filter_state = FilterState::new(filter_line_edit.as_mut().unwrap().text().to_std_string(), column_selector.as_mut().unwrap().current_index(), case_sensitive_button.as_mut().unwrap().is_checked()); }
        }
    }
}
