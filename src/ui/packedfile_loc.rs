//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this file are all the helper functions used by the UI when decoding Loc PackedFiles.

use qt_widgets::abstract_item_view::ScrollMode;
use qt_widgets::action::Action;
use qt_widgets::combo_box::ComboBox;
use qt_widgets::header_view::ResizeMode;
use qt_widgets::file_dialog::FileDialog;
use qt_widgets::label::Label;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::menu::Menu;
use qt_widgets::slots::{SlotQtCorePointRef, SlotCIntQtCoreQtSortOrder};
use qt_widgets::table_view::TableView;
use qt_widgets::widget::Widget;

use qt_gui::brush::Brush;
use qt_gui::cursor::Cursor;
use qt_gui::gui_application::GuiApplication;
use qt_gui::key_sequence::KeySequence;
use qt_gui::list::ListStandardItemMutPtr;
use qt_gui::slots::{SlotStandardItemMutPtr, SlotCIntCIntCInt};
use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::sort_filter_proxy_model::SortFilterProxyModel;
use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::object::Object;
use qt_core::connection::Signal;
use qt_core::signal_blocker::SignalBlocker;
use qt_core::item_selection::ItemSelection;
use qt_core::item_selection_model::SelectionFlag;
use qt_core::variant::Variant;
use qt_core::slots::{SlotBool, SlotCInt, SlotStringRef, SlotItemSelectionRefItemSelectionRef, SlotModelIndexRefModelIndexRefVectorVectorCIntRef};
use qt_core::reg_exp::RegExp;
use qt_core::qt::{Orientation, CheckState, ContextMenuPolicy, ShortcutContext, SortOrder, CaseSensitivity, GlobalColor, MatchFlag};

use std::collections::BTreeMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver};

use crate::TABLE_STATES_UI;
use crate::AppUI;
use crate::Commands;
use crate::Data;
use crate::QString;
use crate::common::*;
use crate::common::communications::*;
use crate::error::Result;
use crate::ui::*;
use crate::ui::table_state::*;

/// Struct `PackedFileLocTreeView`: contains all the stuff we need to give to the program to show a
/// `TreeView` with the data of a Loc PackedFile, allowing us to manipulate it.
pub struct PackedFileLocTreeView {
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
    pub slot_context_menu_apply_prefix_to_selection: SlotBool<'static>,
    pub slot_context_menu_clone: SlotBool<'static>,
    pub slot_context_menu_clone_and_append: SlotBool<'static>,
    pub slot_context_menu_copy: SlotBool<'static>,
    pub slot_context_menu_copy_as_lua_table: SlotBool<'static>,
    pub slot_context_menu_paste_in_selection: SlotBool<'static>,
    pub slot_context_menu_paste_as_new_lines: SlotBool<'static>,
    pub slot_context_menu_paste_to_fill_selection: SlotBool<'static>,
    pub slot_context_menu_search: SlotBool<'static>,
    pub slot_context_menu_import: SlotBool<'static>,
    pub slot_context_menu_export: SlotBool<'static>,
    pub slot_smart_delete: SlotBool<'static>,
    pub slots_hide_show_column: Vec<SlotBool<'static>>,

    pub slot_update_search_stuff: SlotNoArgs<'static>,
    pub slot_search: SlotNoArgs<'static>,
    pub slot_prev_match: SlotNoArgs<'static>,
    pub slot_next_match: SlotNoArgs<'static>,
    pub slot_close_search: SlotNoArgs<'static>,
    pub slot_replace_current: SlotNoArgs<'static>,
    pub slot_replace_all: SlotNoArgs<'static>,
}

/// Implementation of PackedFileLocTreeView.
impl PackedFileLocTreeView {

    /// This function creates a new TreeView with the PackedFile's View as father and returns a
    /// `PackedFileLocTreeView` with all his data.
    pub fn create_tree_view(
        sender_qt: &Sender<Commands>,
        sender_qt_data: &Sender<Data>,
        receiver_qt: &Rc<RefCell<Receiver<Data>>>,
        is_modified: &Rc<RefCell<bool>>,
        app_ui: &AppUI,
        layout: *mut GridLayout,
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        global_search_explicit_paths: &Rc<RefCell<Vec<Vec<String>>>>,
        update_global_search_stuff: *mut Action,
        table_state_data: &Rc<RefCell<BTreeMap<Vec<String>, TableStateData>>>,
    ) -> Result<Self> {

        // Send the index back to the background thread, and wait until we get a response.
        sender_qt.send(Commands::DecodePackedFileLoc).unwrap();
        sender_qt_data.send(Data::VecString(packed_file_path.borrow().to_vec())).unwrap();
        let packed_file_data = match check_message_validity_recv2(&receiver_qt) { 
            Data::Loc(data) => Rc::new(RefCell::new(data)),
            Data::Error(error) => return Err(error),
            _ => panic!(THREADS_MESSAGE_ERROR), 
        };

        // Create the "Undo" stuff needed for the Undo/Redo functions to work.
        let undo_lock = Rc::new(RefCell::new(false));
        let undo_redo_enabler = Action::new(()).into_raw();
        if table_state_data.borrow().get(&*packed_file_path.borrow()).is_none() {
            let _y = table_state_data.borrow_mut().insert(packed_file_path.borrow().to_vec(), TableStateData::new_empty());
        }

        // Create the TableView.
        let table_view = TableView::new().into_raw();
        let filter_model = SortFilterProxyModel::new().into_raw();
        let model = StandardItemModel::new(()).into_raw();

        // Make the last column fill all the available space, if the setting says so.
        if SETTINGS.lock().unwrap().settings_bool["extend_last_column_on_tables"] { 
            unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }
        }

        // Create the filter's LineEdit.
        let row_filter_line_edit = LineEdit::new(()).into_raw();
        unsafe { row_filter_line_edit.as_mut().unwrap().set_placeholder_text(&QString::from_std_str("Type here to filter the rows in the table. Works with Regex too!")); }

        // Create the filter's column selector.
        let row_filter_column_selector = ComboBox::new().into_raw();
        let row_filter_column_list = StandardItemModel::new(()).into_raw();
        unsafe { row_filter_column_selector.as_mut().unwrap().set_model(row_filter_column_list as *mut AbstractItemModel); }
        unsafe { row_filter_column_selector.as_mut().unwrap().add_item(&QString::from_std_str("Key")); }
        unsafe { row_filter_column_selector.as_mut().unwrap().add_item(&QString::from_std_str("Text")); }

        // Create the filter's "Case Sensitive" button.
        let row_filter_case_sensitive_button = PushButton::new(&QString::from_std_str("Case Sensitive")).into_raw();
        unsafe { row_filter_case_sensitive_button.as_mut().unwrap().set_checkable(true); }

        // Prepare the TableView to have a Contextual Menu.
        unsafe { table_view.as_mut().unwrap().set_context_menu_policy(ContextMenuPolicy::Custom); }
        unsafe { table_view.as_mut().unwrap().set_horizontal_scroll_mode(ScrollMode::Pixel); }
        
        // Enable sorting the columns.
        unsafe { table_view.as_mut().unwrap().set_sorting_enabled(true); }
        unsafe { table_view.as_mut().unwrap().sort_by_column((-1, SortOrder::Ascending)); }
        unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_sections_movable(true); }
        unsafe { table_view.as_mut().unwrap().set_alternating_row_colors(true); };

        // Load the data to the Table. For some reason, if we do this after setting the titles of
        // the columns, the titles will be reseted to 1, 2, 3,... so we do this here.
        Self::load_data_to_tree_view(&packed_file_data.borrow(), model);

        // Configure the table to fit Loc PackedFiles.
        unsafe { table_view.as_mut().unwrap().vertical_header().as_mut().unwrap().set_visible(true); }
        unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_visible(true); }

        // Add Table to the Grid.
        unsafe { filter_model.as_mut().unwrap().set_source_model(model as *mut AbstractItemModel); }
        unsafe { table_view.as_mut().unwrap().set_model(filter_model as *mut AbstractItemModel); }
        unsafe { layout.as_mut().unwrap().add_widget((table_view as *mut Widget, 0, 0, 1, 3)); }
        unsafe { layout.as_mut().unwrap().add_widget((row_filter_line_edit as *mut Widget, 2, 0, 1, 1)); }
        unsafe { layout.as_mut().unwrap().add_widget((row_filter_case_sensitive_button as *mut Widget, 2, 1, 1, 1)); }
        unsafe { layout.as_mut().unwrap().add_widget((row_filter_column_selector as *mut Widget, 2, 2, 1, 1)); }

        // Create the main search widget.
        let search_widget = Widget::new().into_raw();

        // Create the "Search" Grid and his internal widgets.
        let grid = GridLayout::new().into_raw();
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
        column_selector.add_item(&QString::from_std_str("Key"));
        column_selector.add_item(&QString::from_std_str("Text"));
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
        unsafe { search_widget.as_mut().unwrap().set_layout(grid as *mut Layout); }
        unsafe { layout.as_mut().unwrap().add_widget((search_widget as *mut Widget, 1, 0, 1, 3)); }
        unsafe { search_widget.as_mut().unwrap().hide(); }

        // Store the search results and the currently selected search item.
        let matches: Rc<RefCell<BTreeMap<ModelIndexWrapped, Option<ModelIndexWrapped>>>> = Rc::new(RefCell::new(BTreeMap::new()));
        let position: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));

        // The data here represents "pattern", "flags to search", "column (-1 for all)".
        let search_data: Rc<RefCell<(String, Flags<MatchFlag>, i32)>> = Rc::new(RefCell::new(("".to_owned(), Flags::from_enum(MatchFlag::Contains), -1)));

        // Action to update the search stuff when needed.
        let update_search_stuff = Action::new(()).into_raw();

        // Build the columns. If we have a model from before, use it to paint our cells as they were last time we painted them.
        build_columns(table_view, model);

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
        let context_menu_apply_prefix_to_selection = context_menu_apply_submenu.add_action(&QString::from_std_str("&Apply Prefix to Selection"));

        let mut context_menu_clone_submenu = Menu::new(&QString::from_std_str("&Clone..."));
        let context_menu_clone = context_menu_clone_submenu.add_action(&QString::from_std_str("&Clone and Insert"));
        let context_menu_clone_and_append = context_menu_clone_submenu.add_action(&QString::from_std_str("Clone and &Append"));

        let mut context_menu_copy_submenu = Menu::new(&QString::from_std_str("&Copy..."));
        let context_menu_copy = context_menu_copy_submenu.add_action(&QString::from_std_str("&Copy"));
        let context_menu_copy_as_lua_table = context_menu_copy_submenu.add_action(&QString::from_std_str("Copy as &LUA Table"));

        let mut context_menu_paste_submenu = Menu::new(&QString::from_std_str("&Paste..."));
        let context_menu_paste_in_selection = context_menu_paste_submenu.add_action(&QString::from_std_str("&Paste in Selection"));
        let context_menu_paste_as_new_lines = context_menu_paste_submenu.add_action(&QString::from_std_str("&Paste as New Rows"));
        let context_menu_paste_to_fill_selection = context_menu_paste_submenu.add_action(&QString::from_std_str("&Paste to Fill Selection"));

        let context_menu_search = context_menu.add_action(&QString::from_std_str("&Search"));

        let context_menu_import = context_menu.add_action(&QString::from_std_str("&Import"));
        let context_menu_export = context_menu.add_action(&QString::from_std_str("&Export"));

        let context_menu_hide_show_submenu = Menu::new(&QString::from_std_str("&Hide/Show...")).into_raw();

        let context_menu_undo = context_menu.add_action(&QString::from_std_str("&Undo"));
        let context_menu_redo = context_menu.add_action(&QString::from_std_str("&Redo"));

        // Set the shortcuts for these actions.
        unsafe { context_menu_add.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_loc["add_row"]))); }
        unsafe { context_menu_insert.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_loc["insert_row"]))); }
        unsafe { context_menu_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_loc["delete_row"]))); }
        unsafe { context_menu_apply_prefix_to_selection.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_loc["apply_prefix_to_selection"]))); }        
        unsafe { context_menu_clone.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_loc["clone_row"]))); }
        unsafe { context_menu_clone_and_append.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_loc["clone_and_append_row"]))); }
        unsafe { context_menu_copy.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_loc["copy"]))); }
        unsafe { context_menu_copy_as_lua_table.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_loc["copy_as_lua_table"]))); }
        unsafe { context_menu_paste_in_selection.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_loc["paste_in_selection"]))); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_loc["paste_as_new_row"]))); }
        unsafe { context_menu_paste_to_fill_selection.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_loc["paste_to_fill_selection"]))); }
        unsafe { context_menu_search.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_loc["search"]))); }
        unsafe { context_menu_import.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_loc["import_tsv"]))); }
        unsafe { context_menu_export.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_loc["export_tsv"]))); }
        unsafe { smart_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_loc["smart_delete"]))); }
        unsafe { context_menu_undo.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_loc["undo"]))); }
        unsafe { context_menu_redo.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().packed_files_loc["redo"]))); }

        // Set the shortcuts to only trigger in the Table.
        unsafe { context_menu_add.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_insert.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_apply_prefix_to_selection.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_clone.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_clone_and_append.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_copy.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_copy_as_lua_table.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_paste_in_selection.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_paste_to_fill_selection.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_search.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_import.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_export.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { smart_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_undo.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_redo.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }

        // Add the actions to the TableView, so the shortcuts work.
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_add); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_insert); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_delete); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_apply_prefix_to_selection); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_clone); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_clone_and_append); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_copy); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_copy_as_lua_table); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_paste_in_selection); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_paste_as_new_lines); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_paste_to_fill_selection); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_search); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_import); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_export); }
        unsafe { table_view.as_mut().unwrap().add_action(smart_delete); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_undo); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_redo); }

        // Status Tips for the actions.
        unsafe { context_menu_add.as_mut().unwrap().set_status_tip(&QString::from_std_str("Add an empty row at the end of the table.")); }
        unsafe { context_menu_insert.as_mut().unwrap().set_status_tip(&QString::from_std_str("Insert an empty row just above the one selected.")); }
        unsafe { context_menu_delete.as_mut().unwrap().set_status_tip(&QString::from_std_str("Delete all the selected rows.")); }
        unsafe { context_menu_apply_prefix_to_selection.as_mut().unwrap().set_status_tip(&QString::from_std_str("Apply a prefix to every cell in the selected cells.")); }
        unsafe { context_menu_clone.as_mut().unwrap().set_status_tip(&QString::from_std_str("Duplicate the selected rows and insert the new rows under the original ones.")); }
        unsafe { context_menu_clone_and_append.as_mut().unwrap().set_status_tip(&QString::from_std_str("Duplicate the selected rows and append the new rows at the end of the table.")); }
        unsafe { context_menu_copy.as_mut().unwrap().set_status_tip(&QString::from_std_str("Copy whatever is selected to the Clipboard.")); }
        unsafe { context_menu_copy_as_lua_table.as_mut().unwrap().set_status_tip(&QString::from_std_str("Turns the entire Loc PackedFile into a LUA Table and copies it to the clipboard.")); }
        unsafe { context_menu_paste_in_selection.as_mut().unwrap().set_status_tip(&QString::from_std_str("Try to paste whatever is in the Clipboard. Does nothing if the data is not compatible with the cell.")); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().set_status_tip(&QString::from_std_str("Try to paste whatever is in the Clipboard as new lines at the end of the table. Does nothing if the data is not compatible with the cell.")); }
        unsafe { context_menu_paste_to_fill_selection.as_mut().unwrap().set_status_tip(&QString::from_std_str("Try to paste whatever is in the Clipboard in EVERY CELL selected. Does nothing if the data is not compatible with the cell.")); }
        unsafe { context_menu_search.as_mut().unwrap().set_status_tip(&QString::from_std_str("Search what you want in the table. Also allows you to replace coincidences.")); }
        unsafe { context_menu_import.as_mut().unwrap().set_status_tip(&QString::from_std_str("Import a TSV file into this table, replacing all the data.")); }
        unsafe { context_menu_export.as_mut().unwrap().set_status_tip(&QString::from_std_str("Export this table's data into a TSV file.")); }
        unsafe { context_menu_undo.as_mut().unwrap().set_status_tip(&QString::from_std_str("A classic.")); }
        unsafe { context_menu_redo.as_mut().unwrap().set_status_tip(&QString::from_std_str("Another classic.")); }

        // Insert some separators to space the menu, and the paste submenu.
        unsafe { context_menu.insert_separator(context_menu_copy); }
        unsafe { context_menu.insert_menu(context_menu_search, context_menu_apply_submenu.into_raw()); }
        unsafe { context_menu.insert_menu(context_menu_search, context_menu_clone_submenu.into_raw()); }
        unsafe { context_menu.insert_menu(context_menu_search, context_menu_copy_submenu.into_raw()); }
        unsafe { context_menu.insert_menu(context_menu_search, context_menu_paste_submenu.into_raw()); }
        unsafe { context_menu.insert_separator(context_menu_search); }
        unsafe { context_menu.insert_separator(context_menu_import); }
        unsafe { context_menu.insert_separator(context_menu_undo); }
        unsafe { context_menu.insert_menu(context_menu_undo, context_menu_hide_show_submenu); }
        unsafe { context_menu.insert_separator(context_menu_undo); }

        // Create the "Hide/Show" slots and actions and connect them.
        let mut slots_hide_show_column = vec![];
        let mut actions_hide_show_column = vec![];
        for column in 0..3 {

            let hide_show_slot = SlotBool::new(clone!(
                packed_file_path => move |state| {
                    unsafe { table_view.as_mut().unwrap().set_column_hidden(column as i32, state); }

                    // Update the state of the column in the table history.
                    if let Some(history_state) = TABLE_STATES_UI.lock().unwrap().get_mut(&packed_file_path.borrow().to_vec()) {
                        if state {
                            if !history_state.columns_state.hidden_columns.contains(&(column as i32)) {
                                history_state.columns_state.hidden_columns.push(column as i32);
                            }
                        }
                        else {
                            let pos = history_state.columns_state.hidden_columns.iter().position(|x| *x as usize == column).unwrap();
                            history_state.columns_state.hidden_columns.remove(pos);
                        }
                    }
                }
            ));
            let name = if column == 0 { "Key" } else if column == 1 { "Text" } else { "Tooltip" };
            let hide_show_action = unsafe { context_menu_hide_show_submenu.as_mut().unwrap().add_action(&QString::from_std_str(&name)) };
            unsafe { hide_show_action.as_mut().unwrap().set_checkable(true); }
            unsafe { hide_show_action.as_mut().unwrap().signals().toggled().connect(&hide_show_slot); }

            slots_hide_show_column.push(hide_show_slot);
            actions_hide_show_column.push(hide_show_action);
        }

        // Slots for the TableView...
        let slots = Self {
            slot_column_moved: SlotCIntCIntCInt::new(clone!(
                packed_file_path => move |_, visual_base, visual_new| {
                    if let Some(state) = TABLE_STATES_UI.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
                        state.columns_state.visual_order.push((visual_base, visual_new));
                    }
                }
            )),

            slot_sort_order_column_changed: SlotCIntQtCoreQtSortOrder::new(clone!(
                packed_file_path => move |column, order| {
                    if let Some(state) = TABLE_STATES_UI.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
                        state.columns_state.sorting_column = (column, if let SortOrder::Ascending = order { false } else { true });
                    }
                }
            )),

            slot_undo: SlotNoArgs::new(clone!(
                global_search_explicit_paths,
                packed_file_path,
                app_ui,
                is_modified,
                packed_file_data,
                table_state_data,
                sender_qt,
                sender_qt_data,
                undo_lock,
                table_state_data => move || {
                    {
                        let mut table_state_data = table_state_data.borrow_mut();
                        let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                        Self::undo_redo(
                            &app_ui,
                            &sender_qt,
                            &sender_qt_data,
                            &is_modified,
                            &packed_file_path,
                            &packed_file_data,
                            table_view,
                            model,
                            filter_model,
                            &mut table_state_data.undo_history,
                            &mut table_state_data.redo_history,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
                            &undo_lock,
                        );

                        update_undo_model(model, table_state_data.undo_model); 
                    }
                    unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                    unsafe { update_search_stuff.as_mut().unwrap().trigger(); }
                }
            )),

            slot_redo: SlotNoArgs::new(clone!(
                global_search_explicit_paths,
                packed_file_path,
                app_ui,
                is_modified,
                packed_file_data,
                table_state_data,
                sender_qt,
                sender_qt_data,
                undo_lock,
                table_state_data => move || {
                    {
                        let mut table_state_data = table_state_data.borrow_mut();
                        let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                        Self::undo_redo(
                            &app_ui,
                            &sender_qt,
                            &sender_qt_data,
                            &is_modified,
                            &packed_file_path,
                            &packed_file_data,
                            table_view,
                            model,
                            filter_model,
                            &mut table_state_data.redo_history,
                            &mut table_state_data.undo_history,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
                            &undo_lock,
                        );

                        update_undo_model(model, table_state_data.undo_model); 
                    }
                    unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                    unsafe { update_search_stuff.as_mut().unwrap().trigger(); }
                }
            )),

            slot_undo_redo_enabler: SlotNoArgs::new(clone!(
                app_ui,
                table_state_data,
                packed_file_path => move || { 
                    let table_state_data = table_state_data.borrow_mut();
                    let table_state_data = table_state_data.get(&*packed_file_path.borrow()).unwrap();
                    unsafe {
                        if table_state_data.undo_history.is_empty() && !table_state_data.is_renamed { 
                            context_menu_undo.as_mut().unwrap().set_enabled(false); 
                            undo_paint_for_packed_file(&app_ui, model, &packed_file_path);
                        }
                        else { context_menu_undo.as_mut().unwrap().set_enabled(true); }

                        if table_state_data.redo_history.is_empty() { context_menu_redo.as_mut().unwrap().set_enabled(false); }
                        else { context_menu_redo.as_mut().unwrap().set_enabled(true); }
                    }
                }
            )),

            slot_context_menu: SlotQtCorePointRef::new(move |_| { context_menu.exec2(&Cursor::pos()); }),
            slot_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(move  |_,_| {

                    // Turns out that this slot doesn't give the the amount of selected items, so we have to get them ourselfs.
                    let indexes = unsafe { table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selected_indexes() };

                    // If we have something selected, enable these actions.
                    if indexes.count(()) > 0 {
                        unsafe {
                            context_menu_clone.as_mut().unwrap().set_enabled(true);
                            context_menu_clone_and_append.as_mut().unwrap().set_enabled(true);
                            context_menu_copy.as_mut().unwrap().set_enabled(true);
                            context_menu_delete.as_mut().unwrap().set_enabled(true);

                            // The "Apply" actions have to be enabled only when all the indexes are valid for the operation. 
                            let mut columns = vec![];
                            for index in 0..indexes.count(()) {
                                let model_index = indexes.at(index);
                                if model_index.is_valid() { columns.push(model_index.column()); }
                            }

                            columns.sort();
                            columns.dedup();

                            let can_apply = if columns.contains(&2) { false } else { true };    
                            context_menu_apply_prefix_to_selection.as_mut().unwrap().set_enabled(can_apply);
                        }
                    }

                    // Otherwise, disable them.
                    else {
                        unsafe {
                            context_menu_apply_prefix_to_selection.as_mut().unwrap().set_enabled(false);
                            context_menu_clone.as_mut().unwrap().set_enabled(false);
                            context_menu_clone_and_append.as_mut().unwrap().set_enabled(false);
                            context_menu_copy.as_mut().unwrap().set_enabled(false);
                            context_menu_delete.as_mut().unwrap().set_enabled(false);
                        }
                    }
                }
            ),
            save_changes: SlotModelIndexRefModelIndexRefVectorVectorCIntRef::new(clone!(
                global_search_explicit_paths,
                packed_file_path,
                app_ui,
                is_modified,
                packed_file_data,
                sender_qt,
                sender_qt_data => move |_,_,roles| {

                    // To avoid doing this multiple times due to the cell painting stuff, we need to check the role.
                    // This has to be allowed ONLY if the role is 0 (DisplayText), 2 (EditorText) or 10 (CheckStateRole).
                    // 16 is a role we use as an special trigger for this.
                    if roles.contains(&0) || roles.contains(&2) || roles.contains(&10) || roles.contains(&16) {

                        // Try to save the PackedFile to the main PackFile.
                        Self::save_to_packed_file(
                            &sender_qt,
                            &sender_qt_data,
                            &is_modified,
                            &app_ui,
                            &packed_file_data,
                            &packed_file_path,
                            model,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
                        );

                        // Update the search stuff, if needed.
                        unsafe { update_search_stuff.as_mut().unwrap().trigger(); }
                    }
                }
            )),
            slot_item_changed: SlotStandardItemMutPtr::new(clone!(
                packed_file_path,
                table_state_data,
                undo_lock => move |item| {

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

                            update_undo_model(model, table_state_data.undo_model); 
                        }

                        unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                    }
                }
            )),

            slot_row_filter_change_text: SlotStringRef::new(clone!(
                packed_file_path => move |filter_text| {
                    filter_table(
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
                    filter_table(
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
                    filter_table(
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
                is_modified,
                packed_file_data,
                table_state_data,
                sender_qt,
                sender_qt_data => move |_| {

                    // Create a new list of StandardItem.
                    let mut qlist = ListStandardItemMutPtr::new(());

                    // Create an empty row.
                    let mut key = StandardItem::new(&QString::from_std_str(""));
                    let mut text = StandardItem::new(&QString::from_std_str(""));
                    let mut tooltip = StandardItem::new(());
                    tooltip.set_editable(false);
                    tooltip.set_checkable(true);
                    tooltip.set_check_state(CheckState::Checked);

                    // Paint the cells.
                    key.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                    text.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                    tooltip.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));

                    // Add an empty row to the list.
                    unsafe { qlist.append_unsafe(&key.into_raw()); }
                    unsafe { qlist.append_unsafe(&text.into_raw()); }
                    unsafe { qlist.append_unsafe(&tooltip.into_raw()); }

                    // Append the new row.
                    unsafe { model.as_mut().unwrap().append_row(&qlist); }

                    // Save, so there are no discrepances between the normal and undo models.
                    Self::save_to_packed_file(
                        &sender_qt,
                        &sender_qt_data,
                        &is_modified,
                        &app_ui,
                        &packed_file_data,
                        &packed_file_path,
                        model,
                        &global_search_explicit_paths,
                        update_global_search_stuff,
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
                is_modified,
                packed_file_data,
                table_state_data,
                sender_qt,
                sender_qt_data => move |_| {

                    // Create a new list of StandardItem.
                    let mut qlist = ListStandardItemMutPtr::new(());

                    // Create an empty row.
                    let mut key = StandardItem::new(&QString::from_std_str(""));
                    let mut text = StandardItem::new(&QString::from_std_str(""));
                    let mut tooltip = StandardItem::new(());
                    tooltip.set_editable(false);
                    tooltip.set_checkable(true);
                    tooltip.set_check_state(CheckState::Checked);

                    // Paint the cells.
                    key.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                    text.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                    tooltip.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));

                    // Add an empty row to the list.
                    unsafe { qlist.append_unsafe(&key.into_raw()); }
                    unsafe { qlist.append_unsafe(&text.into_raw()); }
                    unsafe { qlist.append_unsafe(&tooltip.into_raw()); }

                    // Get the current row and insert the new one.
                    let indexes = unsafe { filter_model.as_mut().unwrap().map_selection_to_source(&table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };
                    let row = if indexes.count(()) > 0 {
                        let model_index = indexes.at(0);
                        if model_index.is_valid() {
                            unsafe { model.as_mut().unwrap().insert_row((model_index.row(), &qlist)); }
                            model_index.row()
                        } else { return }
                    }

                    // Otherwise, just do the same the "Add Row" do.
                    else { 
                        unsafe { model.as_mut().unwrap().append_row(&qlist); } 
                        unsafe { model.as_mut().unwrap().row_count(()) - 1 }
                    };

                    // Save, so there are no discrepances between the normal and undo models.
                    Self::save_to_packed_file(
                        &sender_qt,
                        &sender_qt_data,
                        &is_modified,
                        &app_ui,
                        &packed_file_data,
                        &packed_file_path,
                        model,
                        &global_search_explicit_paths,
                        update_global_search_stuff,
                    );

                    // Update the search stuff, if needed.
                    unsafe { update_search_stuff.as_mut().unwrap().trigger(); }

                    {
                        let mut table_state_data = table_state_data.borrow_mut();
                        let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                        table_state_data.undo_history.push(TableOperations::AddRows(vec![row; 1]));
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
                is_modified,
                packed_file_data,
                table_state_data,
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

                    // If we deleted something, save the PackedFile to the main PackFile.
                    if !rows.is_empty() {
                        Self::save_to_packed_file(
                            &sender_qt,
                            &sender_qt_data,
                            &is_modified,
                            &app_ui,
                            &packed_file_data,
                            &packed_file_path,
                            model,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
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

            slot_context_menu_apply_prefix_to_selection: SlotBool::new(clone!(
                packed_file_path,
                table_state_data,
                app_ui => move |_| {

                    // If we got a prefix, get all the cells in the selection, try to apply it to them.
                    if let Some(mut prefix) = create_apply_prefix_dialog(&app_ui) {

                        // For some reason Qt adds & sometimes, ro remove it if you found it.
                        if let Some(index) = prefix.find('&') { prefix.remove(index); }

                        let mut results = vec![];
                        let indexes = unsafe { filter_model.as_mut().unwrap().map_selection_to_source(&table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };
                        for index in 0..indexes.count(()) {
                            let model_index = indexes.at(index);
                            if model_index.is_valid() { 

                                let text = unsafe { model.as_ref().unwrap().item_from_index(model_index).as_ref().unwrap().text().to_std_string() };
                                let result = format!("{}{}", prefix, text);
                                results.push(result);
                            }
                        }

                        // Then iterate again over every cell applying the new value.
                        for index in 0..indexes.count(()) {
                            let model_index = indexes.at(index);
                            unsafe { model.as_mut().unwrap().item_from_index(model_index).as_mut().unwrap().set_text(&QString::from_std_str(&results[index as usize])) };
                        }

                        {
                            let mut table_state_data = table_state_data.borrow_mut();
                            let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();

                            // If we finished appling prefixes, fix the undo history to have all the previous changes merged into one.
                            // Keep in mind that `None` results should be ignored here.
                            let len = table_state_data.undo_history.len();
                            let mut edits_data = vec![];
                            
                            {
                                let mut edits = table_state_data.undo_history.drain((len - results.len())..);
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
                is_modified,
                packed_file_data,
                table_state_data,
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
                        for column in 0..3 {

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
                            &is_modified,
                            &app_ui,
                            &packed_file_data,
                            &packed_file_path,
                            model,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
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
                is_modified,
                packed_file_data,
                table_state_data,
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
                        for column in 0..3 {

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
                            &is_modified,
                            &app_ui,
                            &packed_file_data,
                            &packed_file_path,
                            model,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
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

                // Create a string to keep all the values in a TSV format (x\tx\tx).
                let mut copy = String::new();

                // Get the current selection.
                let indexes = unsafe { filter_model.as_mut().unwrap().map_selection_to_source(&table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };
                let mut indexes_sorted = vec![];
                for index in 0..indexes.count(()) {
                    indexes_sorted.push(indexes.at(index))
                }

                // Sort the indexes so they follow the visual index, not their logical one. This should fix situations like copying a row and getting a different order in the cells.
                let header = unsafe { table_view.as_ref().unwrap().horizontal_header().as_ref().unwrap() };
                indexes_sorted.sort_unstable_by(|a, b| {
                    if a.row() == b.row() {
                        if header.visual_index(a.column()) < header.visual_index(b.column()) { Ordering::Less }
                        else { Ordering::Greater }
                    } 
                    else if a.row() < b.row() { Ordering::Less }
                    else { Ordering::Greater }
                });

                // Build the copy String.
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
                packed_file_data => move |_| {

                    // We form a "Map<String, Map<String, Any>>" using the key column as Key of the map.
                    let mut lua_table = String::new();
                    lua_table.push_str("LOC = {\n");
                    for entry in &packed_file_data.borrow().entries {
                        lua_table.push_str(&format!("\t[key] = {{"));
                        lua_table.push_str(&format!(" [\"key\"] = {},", format!("\"{}\"", entry.key.replace('\\', "\\\\").replace('\"', "\\\""))));
                        lua_table.push_str(&format!(" [\"text\"] = {},", format!("\"{}\"", entry.text.replace('\\', "\\\\").replace("\"", "\\\""))));
                        lua_table.push_str(&format!(" [\"tooltip\"] = {},", if entry.tooltip { "true" } else { "false" }));

                        // Take out the last comma and close the row.
                        lua_table.pop();
                        lua_table.push_str(" },\n");
                    }

                    // When we finish, we have to remove the last two chars to remove the comma, and close the table.
                    lua_table.pop();
                    lua_table.pop();
                    lua_table.push_str("\n}");

                    // Put the baby into the oven.
                    unsafe { GuiApplication::clipboard().as_mut().unwrap().set_text(&QString::from_std_str(lua_table)); }
                }
            )),

            // NOTE: Saving is not needed in this slot, as this gets detected by the main saving slot.
            // It's needed, however, to deal in a special way here with the undo system.
            slot_context_menu_paste_in_selection: SlotBool::new(clone!(
                packed_file_path,
                table_state_data => move |_| {

                    // If whatever it's in the Clipboard is pasteable in our selection...
                    if check_clipboard(table_view, model, filter_model) {

                        // Get the text from the clipboard and the list of cells to paste to.
                        let clipboard = GuiApplication::clipboard();
                        let mut text = unsafe { clipboard.as_mut().unwrap().text(()).to_std_string() };
                        let indexes = unsafe { filter_model.as_mut().unwrap().map_selection_to_source(&table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };
                        let mut indexes_sorted = vec![];
                        for index in 0..indexes.count(()) {
                            indexes_sorted.push(indexes.at(index))
                        }

                        // Sort the indexes so they follow the visual index, not their logical one. This should fix situations like copying a row and getting a different order in the cells.
                        let header = unsafe { table_view.as_ref().unwrap().horizontal_header().as_ref().unwrap() };
                        indexes_sorted.sort_unstable_by(|a, b| {
                            if a.row() == b.row() {
                                if header.visual_index(a.column()) < header.visual_index(b.column()) { Ordering::Less }
                                else { Ordering::Greater }
                            } 
                            else if a.row() < b.row() { Ordering::Less }
                            else { Ordering::Greater }
                        });

                        // If the text ends in \n, remove it. Excel things. We don't use newlines, so replace them with '\t'.
                        if text.ends_with('\n') { text.pop(); }
                        let text = text.replace('\n', "\t");
                        let text = text.split('\t').collect::<Vec<&str>>();

                        // Get the list of items selected in a format we can deal with easely.
                        let mut items = vec![];
                        for model_index in &indexes_sorted {
                            if model_index.is_valid() {
                                unsafe { items.push(model.as_mut().unwrap().item_from_index(&model_index)); }
                            }
                        }

                        // Zip together both vectors, so we can paste until one of them ends.
                        let data = items.iter().zip(text);
                        let mut changed_cells = 0;
                        for cell in data.clone() {

                            // Qt doesn't notify when you try to set a value to the same value it has. Two days with this fucking bug.....
                            // so we have to do the same check ourselfs and skip the repeated values.
                            if unsafe { cell.0.as_mut().unwrap().is_checkable() } {
                                let current_value = unsafe { cell.0.as_mut().unwrap().check_state() };
                                let new_value = if cell.1.to_lowercase() == "true" || cell.1 == "1" { CheckState::Checked } else { CheckState::Unchecked };
                                if current_value != new_value { 
                                    unsafe { cell.0.as_mut().unwrap().set_check_state(new_value); }
                                    changed_cells += 1;
                                }
                            }

                            // Otherwise, it's just a string.
                            else { 
                                let current_value = unsafe { cell.0.as_mut().unwrap().text().to_std_string() };
                                if &*current_value != cell.1 {
                                    unsafe { cell.0.as_mut().unwrap().set_text(&QString::from_std_str(cell.1)); }
                                    changed_cells += 1;
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

            slot_context_menu_paste_to_fill_selection: SlotBool::new(clone!(
                packed_file_path,
                table_state_data => move |_| {
                
                    // If whatever it's in the Clipboard is pasteable in our selection...
                    if check_clipboard_to_fill_selection(table_view, model, filter_model) {

                        // Get the text from the clipboard and the list of cells to paste to.
                        let clipboard = GuiApplication::clipboard();
                        let text = unsafe { clipboard.as_mut().unwrap().text(()).to_std_string() };
                        let indexes = unsafe { filter_model.as_mut().unwrap().map_selection_to_source(&table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };

                        let mut changed_cells = 0;
                        for index in 0..indexes.count(()) {
                            let model_index = indexes.at(index);
                            if model_index.is_valid() {

                                // Get our item. If it's checkable, we need to check or uncheck the cell.
                                let item = unsafe { model.as_mut().unwrap().item_from_index(&model_index) };

                                // Qt doesn't notify when you try to set a value to the same value it has. Two days with this fucking bug.....
                                // so we have to do the same check ourselfs and skip the repeated values.
                                if unsafe { item.as_mut().unwrap().is_checkable() } {
                                    let current_value = unsafe { item.as_mut().unwrap().check_state() };
                                    let new_value = if text.to_lowercase() == "true" || text == "1"  { CheckState::Checked } else { CheckState::Unchecked };
                                    if current_value != new_value { 
                                        unsafe { item.as_mut().unwrap().set_check_state(new_value); }
                                        changed_cells += 1;
                                    }
                                }

                                // Otherwise, it's just a string.
                                else { 
                                    let current_value = unsafe { item.as_mut().unwrap().text().to_std_string() };
                                    if *current_value != text {
                                        unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str(&text)); }
                                        changed_cells += 1;
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

            slot_context_menu_paste_as_new_lines: SlotBool::new(clone!(
                global_search_explicit_paths,
                packed_file_path,
                app_ui,
                is_modified,
                packed_file_data,
                table_state_data,
                sender_qt,
                sender_qt_data => move |_| {

                    // If whatever it's in the Clipboard is pasteable...
                    if check_clipboard_append_rows(table_view) {

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
                            let mut item = StandardItem::new(());

                            // If the column is "Tooltip", use a bool.
                            let column_logical_index = unsafe { table_view.as_ref().unwrap().horizontal_header().as_ref().unwrap().logical_index(column) };
                            if column_logical_index == 2 {
                                item.set_editable(false);
                                item.set_checkable(true);
                                item.set_check_state(if cell.to_lowercase() == "true" || *cell == "1" { CheckState::Checked } else { CheckState::Unchecked });
                                item.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                            }

                            // Otherwise, create a normal cell.
                            else {
                                item.set_text(&QString::from_std_str(cell));
                                item.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                            }

                            // Add the item to the list.
                            qlist_unordered.push((column_logical_index, item.into_raw()));

                            // If we are in the last column, append the list to the Table and reset it.
                            if column == 2 {
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

                            // If we lack two columns, we have to check all the possible combinations.
                            if column == 1 {

                                // In case our table is "XXX, Tooltip, XXX"...
                                let column_logical_index = unsafe { table_view.as_ref().unwrap().horizontal_header().as_ref().unwrap().logical_index(column) };
                                if column_logical_index == 2 {

                                    // Add the tooltip column.
                                    let mut item = StandardItem::new(());
                                    item.set_editable(false);
                                    item.set_checkable(true);
                                    item.set_check_state(CheckState::Checked);
                                    item.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                    qlist_unordered.push((column_logical_index, item.into_raw()));
                                    
                                    column += 1;
                                    let column_logical_index = unsafe { table_view.as_ref().unwrap().horizontal_header().as_ref().unwrap().logical_index(column) };
                                    let mut item = StandardItem::new(&QString::from_std_str(""));
                                    item.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                    qlist_unordered.push((column_logical_index, item.into_raw()));
                                } 

                                else {
                                    let mut item = StandardItem::new(&QString::from_std_str(""));
                                    item.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                    qlist_unordered.push((column_logical_index, item.into_raw()));

                                    // In case our table is "XXX, XXX, Tooltip"...
                                    column += 1;
                                    let column_logical_index = unsafe { table_view.as_ref().unwrap().horizontal_header().as_ref().unwrap().logical_index(column) };
                                    if column_logical_index == 2 {
                                        let mut item = StandardItem::new(());
                                        item.set_editable(false);
                                        item.set_checkable(true);
                                        item.set_check_state(CheckState::Checked);
                                        item.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                        qlist_unordered.push((column_logical_index, item.into_raw()));  
                                    }

                                    // In case our table is "Tooltip, XXX, XXX"...
                                    else {
                                        let mut item = StandardItem::new(&QString::from_std_str(""));
                                        item.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                        qlist_unordered.push((column_logical_index, item.into_raw()));
                                    }
                                }
                            }

                            // Otherwise, we just lack tooltip.
                            else {

                                let column_logical_index = unsafe { table_view.as_ref().unwrap().horizontal_header().as_ref().unwrap().logical_index(column) };
                                if column_logical_index == 2 { 

                                    // Add the tooltip column.
                                    let mut item = StandardItem::new(());
                                    item.set_editable(false);
                                    item.set_checkable(true);
                                    item.set_check_state(CheckState::Checked);
                                    item.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                    qlist_unordered.push((column_logical_index, item.into_raw()));
                                } 

                                else { 
                                    let mut item = StandardItem::new(&QString::from_std_str(""));
                                    item.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                    qlist_unordered.push((column_logical_index, item.into_raw()));
                                }
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
                                &is_modified,
                                &app_ui,
                                &packed_file_data,
                                &packed_file_path,
                                model,
                                &global_search_explicit_paths,
                                update_global_search_stuff,
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

            slot_context_menu_search: SlotBool::new(move |_| {
                unsafe {
                    if search_widget.as_mut().unwrap().is_visible() { search_widget.as_mut().unwrap().hide(); } 
                    else { search_widget.as_mut().unwrap().show(); }
                }
            }),

            slot_context_menu_import: SlotBool::new(clone!(
                global_search_explicit_paths,
                app_ui,
                is_modified,
                packed_file_data,
                packed_file_path,
                table_state_data,
                sender_qt,
                sender_qt_data,
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
                        sender_qt.send(Commands::ImportTSVPackedFileLoc).unwrap();
                        sender_qt_data.send(Data::LocPathBuf((packed_file_data.borrow().clone(), path))).unwrap();

                        match check_message_validity_recv2(&receiver_qt) {
                            Data::Loc(new_loc_data) => Self::load_data_to_tree_view(&new_loc_data, model),
                            Data::Error(error) => return show_dialog(app_ui.window, false, error),
                            _ => panic!(THREADS_MESSAGE_ERROR),
                        }

                        // Build the columns.
                        build_columns(table_view, model);
                        if SETTINGS.lock().unwrap().settings_bool["adjust_columns_to_content"] {
                            unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
                        }

                        // Make a copy of the old data for the undo system.
                        let old_data = packed_file_data.borrow().clone();

                        // Save the new PackFile's data.
                        Self::save_to_packed_file(
                            &sender_qt,
                            &sender_qt_data,
                            &is_modified,
                            &app_ui,
                            &packed_file_data,
                            &packed_file_path,
                            model,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
                        );

                        // Update the search stuff, if needed.
                        unsafe { update_search_stuff.as_mut().unwrap().trigger(); }

                        {
                            let mut table_state_data = table_state_data.borrow_mut();
                            let table_state_data = table_state_data.get_mut(&*packed_file_path.borrow()).unwrap();
                            table_state_data.undo_history.push(TableOperations::ImportTSVLOC(old_data));
                            table_state_data.redo_history.clear();
                            update_undo_model(model, table_state_data.undo_model); 
                        }

                        unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                    }
                }
            )),
            slot_context_menu_export: SlotBool::new(clone!(
                app_ui,
                sender_qt,
                sender_qt_data,
                packed_file_data,
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

                    // Run it and, if we receive 1 (Accept), export the Loc PackedFile.
                    if file_dialog.exec() == 1 {

                        let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());
                        sender_qt.send(Commands::ExportTSVPackedFileLoc).unwrap();
                        sender_qt_data.send(Data::LocPathBuf((packed_file_data.borrow().clone(), path))).unwrap();

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
                packed_file_path,
                app_ui,
                is_modified,
                packed_file_data,
                packed_file_path,
                table_state_data,
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

                                // Qt doesn't notify when you try to set a value to the same value it has. Two days with this fucking bug.....
                                // so we have to do the same check ourselfs and skip the repeated values.
                                // If it's checkable, it's the last column.
                                if unsafe { item.as_mut().unwrap().is_checkable() } {
                                    let current_value = unsafe { item.as_mut().unwrap().check_state() };
                                    if current_value != CheckState::Unchecked { 
                                        unsafe { edits.push(((*key, *column), (&*item).clone())); }
                                        unsafe { item.as_mut().unwrap().set_check_state(CheckState::Unchecked); }
                                    }
                                }

                                // Otherwise, it's just a string.
                                else { 
                                    let current_value = unsafe { item.as_mut().unwrap().text().to_std_string() };
                                    if !current_value.is_empty() {
                                        unsafe { edits.push(((*key, *column), (&*item).clone())); }
                                        unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str("")); }
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

                    // If something was deleted, save the changes.
                    if !cells.is_empty() {
                        Self::save_to_packed_file(
                            &sender_qt,
                            &sender_qt_data,
                            &is_modified,
                            &app_ui,
                            &packed_file_data,
                            &packed_file_path,
                            model,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
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

            // This is the list of slots to hide/show columns. Is created before all this, so here we just add it.
            slots_hide_show_column,

            // Slot to close the search widget.
            slot_update_search_stuff: SlotNoArgs::new(clone!(
                matches,
                position,
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

                            // Get all the matches from all the columns. Once you got them, process them and get their ModelIndex.
                            for index in 0..3 {
                                let matches_unprocessed = unsafe { model.as_mut().unwrap().find_items((&QString::from_std_str(text), flags.clone(), index)) };
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
                            for index in 0..3 {
                                
                                // Get all the matches from all the columns. Once you got them, process them and get their ModelIndex.
                                let matches_unprocessed = unsafe { model.as_mut().unwrap().find_items((&text, flags.clone(), index)) };
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
                            let column = match &*column {
                                "key" => 0,
                                "text" => 1,
                                "tooltip" => 2,
                                _ => unreachable!(),
                            };

                            // Once you got them, process them and get their ModelIndex.
                            let matches_unprocessed = unsafe { model.as_mut().unwrap().find_items((&text, flags.clone(), column)) };
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
                    *search_data.borrow_mut() = (text.to_std_string(), flags, 
                        match &*column {
                            "*_(all_columns)" => -1,
                            "key" => 0,
                            "text" => 1,
                            "tooltip" => 2,
                            _ => unreachable!(),
                        }
                    );
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
                            } else { return }
                        } else { return }
                        unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str(&replaced_text)); }

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
                packed_file_path,
                table_state_data,
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
                                    let item = unsafe { model.as_mut().unwrap().item_from_index(model_index) };
                                    let text = unsafe { item.as_mut().unwrap().text().to_std_string() };
                                    positions_and_texts.push(((model_index.row(), model_index.column()), text.replace(&text_source, &text_replace)));
                                } else { return }
                            }
                        }

                        // For each position, get his item and change his text.
                        for (index, data) in positions_and_texts.iter().enumerate() {
                            let item = unsafe { model.as_mut().unwrap().item(((data.0).0, (data.0).1)) };
                            unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str(&data.1)); }

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
        unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().signals().section_moved().connect(&slots.slot_column_moved); }
        unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().signals().sort_indicator_changed().connect(&slots.slot_sort_order_column_changed); }
        unsafe { model.as_mut().unwrap().signals().data_changed().connect(&slots.save_changes); }
        unsafe { model.as_mut().unwrap().signals().item_changed().connect(&slots.slot_item_changed); }
        unsafe { context_menu_add.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_add); }
        unsafe { context_menu_insert.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_insert); }
        unsafe { context_menu_delete.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_delete); }
        unsafe { context_menu_apply_prefix_to_selection.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_apply_prefix_to_selection); }
        unsafe { context_menu_clone.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_clone); }
        unsafe { context_menu_clone_and_append.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_clone_and_append); }
        unsafe { context_menu_copy.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_copy); }
        unsafe { context_menu_copy_as_lua_table.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_copy_as_lua_table); }
        unsafe { context_menu_paste_in_selection.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_paste_in_selection); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_paste_as_new_lines); }
        unsafe { context_menu_paste_to_fill_selection.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_paste_to_fill_selection); }
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
            context_menu_apply_prefix_to_selection.as_mut().unwrap().set_enabled(false);
            context_menu_clone.as_mut().unwrap().set_enabled(false);
            context_menu_clone_and_append.as_mut().unwrap().set_enabled(false);
            context_menu_copy.as_mut().unwrap().set_enabled(false);
            context_menu_copy_as_lua_table.as_mut().unwrap().set_enabled(true);
            context_menu_paste_in_selection.as_mut().unwrap().set_enabled(true);
            context_menu_paste_as_new_lines.as_mut().unwrap().set_enabled(true);
            context_menu_paste_to_fill_selection.as_mut().unwrap().set_enabled(true);
            context_menu_import.as_mut().unwrap().set_enabled(true);
            context_menu_export.as_mut().unwrap().set_enabled(true);
            undo_redo_enabler.as_mut().unwrap().trigger();
        }

        // Trigger the "Enable/Disable" slot every time we change the selection in the TreeView.
        unsafe { table_view.as_mut().unwrap().selection_model().as_ref().unwrap().signals().selection_changed().connect(&slots.slot_context_menu_enabler); }

        // If we got an entry for this PackedFile in the state's history, use it.
        if TABLE_STATES_UI.lock().unwrap().get(&*packed_file_path.borrow()).is_some() {
            if let Some(state_data) = TABLE_STATES_UI.lock().unwrap().get_mut(&*packed_file_path.borrow()) {

                // Block the signals during this, so we don't trigger a borrow error.
                let mut blocker1 = unsafe { SignalBlocker::new(row_filter_line_edit.as_mut().unwrap().static_cast_mut() as &mut Object) };
                let mut blocker2 = unsafe { SignalBlocker::new(row_filter_column_selector.as_mut().unwrap().static_cast_mut() as &mut Object) };
                let mut blocker3 = unsafe { SignalBlocker::new(row_filter_case_sensitive_button.as_mut().unwrap().static_cast_mut() as &mut Object) };
                unsafe { row_filter_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&state_data.filter_state.text)); }
                unsafe { row_filter_column_selector.as_mut().unwrap().set_current_index(state_data.filter_state.column); }
                unsafe { row_filter_case_sensitive_button.as_mut().unwrap().set_checked(state_data.filter_state.is_case_sensitive); }
                blocker1.unblock();
                blocker2.unblock();
                blocker3.unblock();

                // Same with everything inside the search widget.
                let mut blocker1 = unsafe { SignalBlocker::new(search_line_edit.as_mut().unwrap().static_cast_mut() as &mut Object) };
                let mut blocker2 = unsafe { SignalBlocker::new(replace_line_edit.as_mut().unwrap().static_cast_mut() as &mut Object) };
                let mut blocker3 = unsafe { SignalBlocker::new(column_selector.as_mut().unwrap().static_cast_mut() as &mut Object) };
                let mut blocker4 = unsafe { SignalBlocker::new(case_sensitive_button.as_mut().unwrap().static_cast_mut() as &mut Object) };
                unsafe { search_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&state_data.search_state.search_text)); }
                unsafe { replace_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&state_data.search_state.replace_text)); }
                unsafe { column_selector.as_mut().unwrap().set_current_index(state_data.search_state.column); }
                unsafe { case_sensitive_button.as_mut().unwrap().set_checked(state_data.search_state.is_case_sensitive); }
                blocker1.unblock();
                blocker2.unblock();
                blocker3.unblock();
                blocker4.unblock();

                // Same with the columns, if we opted to keep their state.
                let mut blocker1 = unsafe { SignalBlocker::new(table_view.as_mut().unwrap().static_cast_mut() as &mut Object) };
                let mut blocker2 = unsafe { SignalBlocker::new(table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().static_cast_mut() as &mut Object) };
                
                if SETTINGS.lock().unwrap().settings_bool["remember_column_state"] {
                    let sort_order = if state_data.columns_state.sorting_column.1 { SortOrder::Descending } else { SortOrder::Ascending };
                    unsafe { table_view.as_mut().unwrap().sort_by_column((state_data.columns_state.sorting_column.0, sort_order)); }
      
                    for (visual_old, visual_new) in &state_data.columns_state.visual_order {
                        unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().move_section(*visual_old, *visual_new); }
                    }

                    // For this we have to "block" the action before checking it, to avoid borrowmut errors. There is no need to unblock, because the blocker will die after the action.
                    for hidden_column in &state_data.columns_state.hidden_columns {
                        unsafe { table_view.as_mut().unwrap().set_column_hidden(*hidden_column, true); }

                        let mut _blocker = unsafe { SignalBlocker::new(actions_hide_show_column[*hidden_column as usize].as_mut().unwrap() as &mut Object) };
                        unsafe { actions_hide_show_column[*hidden_column as usize].as_mut().unwrap().set_checked(true); }
                    }                     
                }
                else { state_data.columns_state = ColumnsState::new((-1, false), vec![], vec![]); }
                
                blocker1.unblock();
                blocker2.unblock();
            }
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
    pub fn load_data_to_tree_view(
        packed_file_data: &Loc,
        model: *mut StandardItemModel,
    ) {
        // First, we delete all the data from the `ListStore`. Just in case there is something there.
        // This wipes out header information, so remember to run "build_columns" after this.
        unsafe { model.as_mut().unwrap().clear(); }

        // Then we add every line to the ListStore.
        for entry in &packed_file_data.entries {

            // Create a new list of StandardItem.
            let mut qlist = ListStandardItemMutPtr::new(());

            // Create the items of the new row.
            let key = StandardItem::new(&QString::from_std_str(&entry.key));
            let text = StandardItem::new(&QString::from_std_str(&entry.text));
            let mut tooltip = StandardItem::new(());
            tooltip.set_editable(false);
            tooltip.set_checkable(true);
            tooltip.set_check_state(if entry.tooltip { CheckState::Checked } else { CheckState::Unchecked });

            // Add the items to the list.
            unsafe { qlist.append_unsafe(&key.into_raw()); }
            unsafe { qlist.append_unsafe(&text.into_raw()); }
            unsafe { qlist.append_unsafe(&tooltip.into_raw()); }

            // Just append a new row.
            unsafe { model.as_mut().unwrap().append_row(&qlist); }
        }

        // If there are no entries, add an empty row with default values, so Qt shows the table anyway.
        if packed_file_data.entries.is_empty() {

            // Create a new list of StandardItem.
            let mut qlist = ListStandardItemMutPtr::new(());

            // Create the items of the new row.
            let key = StandardItem::new(&QString::from_std_str(""));
            let text = StandardItem::new(&QString::from_std_str(""));
            let mut tooltip = StandardItem::new(());
            tooltip.set_editable(false);
            tooltip.set_checkable(true);
            tooltip.set_check_state(CheckState::Checked);

            // Add the items to the list.
            unsafe { qlist.append_unsafe(&key.into_raw()); }
            unsafe { qlist.append_unsafe(&text.into_raw()); }
            unsafe { qlist.append_unsafe(&tooltip.into_raw()); }

            // Just append a new row.
            unsafe { model.as_mut().unwrap().append_row(&qlist); }

            // Remove the row, so the columns stay.
            unsafe { model.as_mut().unwrap().remove_rows((0, 1)); }
        }
    }

    /// This function returns a LocData with all the stuff in the table.
    pub fn return_data_from_tree_view(
        packed_file_data: &mut Loc,
        model: *mut StandardItemModel,
    ) {

        // Clear the data and, for each row we have, add it to the cleared data.
        packed_file_data.entries.clear();
        let rows = unsafe { model.as_mut().unwrap().row_count(()) };
        for row in 0..rows {
            unsafe {
                packed_file_data.entries.push(
                    LocEntry::new(
                        model.as_mut().unwrap().item((row as i32, 0)).as_mut().unwrap().text().to_std_string(),
                        model.as_mut().unwrap().item((row as i32, 1)).as_mut().unwrap().text().to_std_string(),
                        if model.as_mut().unwrap().item((row as i32, 2)).as_mut().unwrap().check_state() == CheckState::Checked { true } else { false },
                    )
                );
            }
        }
    }

    /// Function to save the data from the current StandardItemModel to the PackFile.
    pub fn save_to_packed_file(
        sender_qt: &Sender<Commands>,
        sender_qt_data: &Sender<Data>,
        is_modified: &Rc<RefCell<bool>>,
        app_ui: &AppUI,
        data: &Rc<RefCell<Loc>>,
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        model: *mut StandardItemModel,
        global_search_explicit_paths: &Rc<RefCell<Vec<Vec<String>>>>,
        update_global_search_stuff: *mut Action,
    ) {
        // Update our LocData.
        Self::return_data_from_tree_view(&mut data.borrow_mut(), model);

        // Tell the background thread to start saving the PackedFile.
        sender_qt.send(Commands::EncodePackedFileLoc).unwrap();
        sender_qt_data.send(Data::LocVecString((data.borrow().clone(), packed_file_path.borrow().to_vec()))).unwrap();

        // Set the mod as "Modified".
        *is_modified.borrow_mut() = set_modified(true, &app_ui, Some(packed_file_path.borrow().to_vec()));
        
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
        sender_qt: &Sender<Commands>,
        sender_qt_data: &Sender<Data>,
        is_modified: &Rc<RefCell<bool>>,
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        data: &Rc<RefCell<Loc>>,
        table_view: *mut TableView,
        model: *mut StandardItemModel,
        filter_model: *mut SortFilterProxyModel,
        history_source: &mut Vec<TableOperations>, 
        history_opposite: &mut Vec<TableOperations>,
        global_search_explicit_paths: &Rc<RefCell<Vec<Vec<String>>>>,
        update_global_search_stuff: *mut Action,
        undo_lock: &Rc<RefCell<bool>>, 
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
                for (index, edit) in editions.iter().enumerate() {
                    let row = (edit.0).0;
                    let column = (edit.0).1;
                    let item = edit.1;
                    unsafe { model.as_mut().unwrap().set_item((row, column, item.clone())); }
                    
                    // Trick to tell the model to update everything.
                    if index == editions.len() - 1 { unsafe { model.as_mut().unwrap().item((row, column)).as_mut().unwrap().set_data((&Variant::new0(()), 16)); }}
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
                    &is_modified,
                    &app_ui,
                    &data,
                    &packed_file_path,
                    model,
                    &global_search_explicit_paths,
                    update_global_search_stuff,
                );
            }

            // NOTE: the rows list must ALWAYS be in 1->9 order. Otherwise this breaks.
            TableOperations::RemoveRows(rows) => {

                // First, we re-insert each pack of rows.
                for row_pack in &rows {
                    for (row, items) in row_pack {
                        let mut qlist = ListStandardItemMutPtr::new(());
                        unsafe { items.iter().for_each(|x| qlist.append_unsafe(x)); }
                        unsafe { model.as_mut().unwrap().insert_row((*row, &qlist)); }
                    }
                }

                // Then, create the "redo" action for this one.
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
                    &is_modified,
                    &app_ui,
                    &data,
                    &packed_file_path,
                    model,
                    &global_search_explicit_paths,
                    update_global_search_stuff,
                );
            }

            // This action is special and we have to manually trigger a save for it.
            TableOperations::ImportTSVLOC(table_data) => {

                // Prepare the redo operation.
                history_opposite.push(TableOperations::ImportTSVLOC(data.borrow().clone()));

                Self::load_data_to_tree_view(&table_data, model);
                build_columns(table_view, model);

                // If we want to let the columns resize themselfs...
                if SETTINGS.lock().unwrap().settings_bool["adjust_columns_to_content"] {
                    unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
                }

                // Try to save the PackedFile to the main PackFile.
                Self::save_to_packed_file(
                    &sender_qt,
                    &sender_qt_data,
                    &is_modified,
                    &app_ui,
                    &data,
                    &packed_file_path,
                    model,
                    &global_search_explicit_paths,
                    update_global_search_stuff,
                );
            }

            // Any other variant is not possible in this kind of table.
            _ => { return },
        }
    }
}

/// This function is meant to be used to prepare and build the column headers, and the column-related stuff.
/// His intended use is for just after we reload the data to the table.
fn build_columns(
    table_view: *mut TableView,
    model: *mut StandardItemModel
) {

    // Fix the columns titles.
    unsafe { model.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Key")))); }
    unsafe { model.as_mut().unwrap().set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Text")))); }
    unsafe { model.as_mut().unwrap().set_header_data((2, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Tooltip")))); }

    unsafe { table_view.as_mut().unwrap().set_column_width(0, 450); }
    unsafe { table_view.as_mut().unwrap().set_column_width(1, 450); }
    unsafe { table_view.as_mut().unwrap().set_column_width(2, 60); }
}

/// This function checks if the data in the clipboard is suitable for the selected Items.
fn check_clipboard(
    table_view: *mut TableView,
    model: *mut StandardItemModel,
    filter_model: *mut SortFilterProxyModel
) -> bool {

    // Get the current selection.
    let clipboard = GuiApplication::clipboard();
    let mut text = unsafe { clipboard.as_mut().unwrap().text(()).to_std_string() };
    let indexes = unsafe { filter_model.as_mut().unwrap().map_selection_to_source(&table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };
    let mut indexes_sorted = vec![];
    for index in 0..indexes.count(()) {
        indexes_sorted.push(indexes.at(index))
    }

    // Sort the indexes so they follow the visual index, not their logical one. This should fix situations like copying a row and getting a different order in the cells.
    let header = unsafe { table_view.as_ref().unwrap().horizontal_header().as_ref().unwrap() };
    indexes_sorted.sort_unstable_by(|a, b| {
        if a.row() == b.row() {
            if header.visual_index(a.column()) < header.visual_index(b.column()) { Ordering::Less }
            else { Ordering::Greater }
        } 
        else if a.row() < b.row() { Ordering::Less }
        else { Ordering::Greater }
    });

    // If there is nothing selected, don't waste your time.
    if indexes_sorted.is_empty() { return false }

    // If the text ends in \n, remove it. Excel things. We don't use newlines, so replace them with '\t'.
    if text.ends_with('\n') { text.pop(); }
    let text = text.replace('\n', "\t");
    let text = text.split('\t').collect::<Vec<&str>>();

    // Get the list of items selected in a format we can deal with easely.
    let mut items = vec![];
    for model_index in &indexes_sorted {
        if model_index.is_valid() {
            unsafe { items.push(model.as_mut().unwrap().item_from_index(&model_index)); }
        }
    }

    // If none of the items are valid, stop.
    if items.is_empty() { return false }

    // Zip together both vectors.
    let data = items.iter().zip(text);
    for cell in data {

        // If it's checkable, we need to see if his text it's a bool. Otherwise, it's just a string.
        if unsafe { cell.0.as_mut().unwrap().is_checkable() } {
            if cell.1.to_lowercase() != "true" && cell.1.to_lowercase() != "false" && cell.1 != "1" && cell.1 != "0" { return false }
        } else { continue }
    }

    // If we reach this place, it means none of the cells was incorrect, so we can paste.
    true
}

/// This function checks if the data in the clipboard is suitable for be pasted in all selected cells.
fn check_clipboard_to_fill_selection(
    table_view: *mut TableView,
    model: *mut StandardItemModel,
    filter_model: *mut SortFilterProxyModel
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
            let item = unsafe { model.as_mut().unwrap().item_from_index(&model_index) };

            // If it's checkable, we need to see if his text it's a bool. Otherwise, it's just a string.
            if unsafe { item.as_mut().unwrap().is_checkable() } {
                if text.to_lowercase() != "true" && text.to_lowercase() != "false" && text != "1" && text != "0" { return false }
            } else { continue }
        }
    }

    // If we reach this place, it means none of the cells was incorrect, so we can paste.
    true
}


/// This function checks if the data in the clipboard is suitable to be appended as rows at the end of the Table.
fn check_clipboard_append_rows(
    table_view: *mut TableView,
) -> bool {

    // Get the text from the clipboard.
    let clipboard = GuiApplication::clipboard();
    let mut text = unsafe { clipboard.as_mut().unwrap().text(()).to_std_string() };

    // If the text ends in \n, remove it. Excel things. We don't use newlines, so replace them with '\t'.
    if text.ends_with('\n') { text.pop(); }
    let text = text.replace('\n', "\t");
    let text = text.split('\t').collect::<Vec<&str>>();

    // Get the index for the column, and the position of the "Tooltip" column.
    let mut column = 0;
    for cell in text {
        
        // If the column is "Tooltip", ensure it's a boolean.
        let column_logical_index = unsafe { table_view.as_ref().unwrap().horizontal_header().as_ref().unwrap().logical_index(column) };
        if column_logical_index == 2 && cell.to_lowercase() != "true" && cell.to_lowercase() != "false" && cell != "1" && cell != "0" { return false }

        // Reset or increase the column count, if needed.
        if column == 2 { column = 0; } else { column += 1; }
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
