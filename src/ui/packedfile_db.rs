// In this file are all the helper functions used by the UI when decoding DB PackedFiles.
extern crate qt_widgets;
extern crate qt_gui;
extern crate qt_core;

use qt_widgets::abstract_item_view::{EditTrigger, ScrollMode, SelectionMode};
use qt_widgets::action::Action;
use qt_widgets::file_dialog::FileDialog;
use qt_widgets::frame::Frame;
use qt_widgets::group_box::GroupBox;
use qt_widgets::header_view::ResizeMode;
use qt_widgets::menu::Menu;
use qt_widgets::label::Label;
use qt_widgets::slots::{SlotQtCorePointRef, SlotCIntQtCoreQtSortOrder};
use qt_widgets::splitter::Splitter;
use qt_widgets::text_edit::TextEdit;
use qt_widgets::table_view::TableView;
use qt_widgets::widget::Widget;

use qt_gui::cursor::Cursor;
use qt_gui::font::{Font, StyleHint };
use qt_gui::font_metrics::FontMetrics;
use qt_gui::gui_application::GuiApplication;
use qt_gui::key_sequence::KeySequence;
use qt_gui::list::ListStandardItemMutPtr;
use qt_gui::slots::{SlotStandardItemMutPtr, SlotCIntCIntCInt};
use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;
use qt_gui::text_char_format::TextCharFormat;
use qt_gui::text_cursor::{MoveOperation, MoveMode};

use qt_core::signal_blocker::SignalBlocker;
use qt_core::sort_filter_proxy_model::SortFilterProxyModel;
use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::connection::Signal;
use qt_core::variant::Variant;
use qt_core::item_selection_model::SelectionFlag;
use qt_core::object::Object;
use qt_core::slots::{SlotBool, SlotCInt, SlotStringRef, SlotItemSelectionRefItemSelectionRef, SlotModelIndexRefModelIndexRefVectorVectorCIntRef};
use qt_core::string_list::StringList;
use qt_core::reg_exp::RegExp;
use qt_core::qt::{Orientation, CheckState, ContextMenuPolicy, ShortcutContext, SortOrder, CaseSensitivity, GlobalColor, MatchFlag};

use std::collections::BTreeMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver};

use HISTORY_TABLE_STATE;
use AppUI;
use Commands;
use Data;
use QString;
use common::*;
use common::communications::*;
use error::{Error, ErrorKind, Result};
use ui::*;
use ui::table_state::*;

/// Struct `PackedFileDBTreeView`: contains all the stuff we need to give to the program to show a
/// TableView with the data of a DB PackedFile, allowing us to manipulate it.
pub struct PackedFileDBTreeView {
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

/// Struct PackedFileDBDecoder: contains all the stuff we need to return to be able to decode DB PackedFiles.
pub struct PackedFileDBDecoder {
    pub slot_hex_view_scroll_sync: SlotCInt<'static>,
    pub slot_hex_view_raw_selection_sync: SlotNoArgs<'static>,
    pub slot_hex_view_decoded_selection_sync: SlotNoArgs<'static>,
    pub slot_hex_view_raw_selection_decoding: SlotNoArgs<'static>,
    pub slot_hex_view_decoded_selection_decoding: SlotNoArgs<'static>,
    pub slot_use_this_bool: SlotNoArgs<'static>,
    pub slot_use_this_float: SlotNoArgs<'static>,
    pub slot_use_this_integer: SlotNoArgs<'static>,
    pub slot_use_this_long_integer: SlotNoArgs<'static>,
    pub slot_use_this_string_u8: SlotNoArgs<'static>,
    pub slot_use_this_string_u16: SlotNoArgs<'static>,
    pub slot_use_this_optional_string_u8: SlotNoArgs<'static>,
    pub slot_use_this_optional_string_u16: SlotNoArgs<'static>,
    pub slot_table_change_field_type: SlotModelIndexRefModelIndexRefVectorVectorCIntRef<'static>,
    pub slot_table_view_context_menu_enabler: SlotItemSelectionRefItemSelectionRef<'static>,
    pub slot_table_view_context_menu: SlotQtCorePointRef<'static>,
    pub slot_table_view_context_menu_move_up: SlotBool<'static>,
    pub slot_table_view_context_menu_move_down: SlotBool<'static>,
    pub slot_table_view_context_menu_delete: SlotBool<'static>,
    pub slot_remove_all_fields: SlotNoArgs<'static>,
    pub slot_save_definition: SlotNoArgs<'static>,
    pub slot_table_view_old_versions_context_menu_enabler: SlotItemSelectionRefItemSelectionRef<'static>,
    pub slot_table_view_old_versions_context_menu: SlotQtCorePointRef<'static>,
    pub slot_table_view_old_versions_context_menu_load: SlotBool<'static>,
    pub slot_table_view_old_versions_context_menu_delete: SlotBool<'static>,
}

/// Struct PackedFileDBDecoderStuff: contains all the ui things from the decoder view, so we can pass the easely.
#[derive(Copy, Clone)]
pub struct PackedFileDBDecoderStuff {
    pub hex_view_index: *mut TextEdit,
    pub hex_view_raw: *mut TextEdit,
    pub hex_view_decoded: *mut TextEdit,
    pub table_view: *mut TableView,
    pub table_model: *mut StandardItemModel,

    pub selection_bool_line_edit: *mut LineEdit,
    pub selection_float_line_edit: *mut LineEdit,
    pub selection_integer_line_edit: *mut LineEdit,
    pub selection_long_integer_line_edit: *mut LineEdit,
    pub selection_string_u8_line_edit: *mut LineEdit,
    pub selection_string_u16_line_edit: *mut LineEdit,
    pub selection_optional_string_u8_line_edit: *mut LineEdit,
    pub selection_optional_string_u16_line_edit: *mut LineEdit,

    pub bool_line_edit: *mut LineEdit,
    pub float_line_edit: *mut LineEdit,
    pub integer_line_edit: *mut LineEdit,
    pub long_integer_line_edit: *mut LineEdit,
    pub string_u8_line_edit: *mut LineEdit,
    pub string_u16_line_edit: *mut LineEdit,
    pub optional_string_u8_line_edit: *mut LineEdit,
    pub optional_string_u16_line_edit: *mut LineEdit,

    pub bool_button: *mut PushButton,
    pub float_button: *mut PushButton,
    pub integer_button: *mut PushButton,
    pub long_integer_button: *mut PushButton,
    pub string_u8_button: *mut PushButton,
    pub string_u16_button: *mut PushButton,
    pub optional_string_u8_button: *mut PushButton,
    pub optional_string_u16_button: *mut PushButton,

    pub table_info_type_decoded_label: *mut Label,
    pub table_info_version_decoded_label: *mut Label,
    pub table_info_entry_count_decoded_label: *mut Label,

    pub table_view_old_versions: *mut TableView,
    pub table_model_old_versions: *mut StandardItemModel,

    pub clear_definition_button: *mut PushButton,
    pub save_button: *mut PushButton,

    pub table_view_context_menu: *mut Menu,
    pub table_view_context_menu_move_up: *mut Action,
    pub table_view_context_menu_move_down: *mut Action,
    pub table_view_context_menu_delete: *mut Action,

    pub table_view_old_versions_context_menu: *mut Menu,
    pub table_view_old_versions_context_menu_load: *mut Action,
    pub table_view_old_versions_context_menu_delete: *mut Action,
}

/// Struct PackedFileDBDecoderStuffNonUI: contains data needed for the decoder to properly work.
#[derive(Clone)]
pub struct PackedFileDBDecoderStuffNonUI {
    pub packed_file_path: Vec<String>,
    pub packed_file_data: Vec<u8>,
    pub initial_index: usize,
    pub version: u32,
    pub entry_count: u32,
}

/// Enum MathOperationResult: used for keeping together different types of results after a math operation.
/// None is an special type, so we know that we have to ignore an specified cell.
#[derive(Clone, PartialEq)]
enum MathOperationResult {
    Float(f32),
    Integer(i32),
    LongInteger(i64),
    None
}

/// Implementation of PackedFileDBTreeView.
impl PackedFileDBTreeView {

    /// This function creates a new Table with the PackedFile's View as father and returns a
    /// `PackedFileDBTreeView` with all his data.
    pub fn create_table_view(
        sender_qt: Sender<Commands>,
        sender_qt_data: &Sender<Data>,
        receiver_qt: &Rc<RefCell<Receiver<Data>>>,
        is_modified: &Rc<RefCell<bool>>,
        app_ui: &AppUI,
        layout: *mut GridLayout,
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        global_search_explicit_paths: &Rc<RefCell<Vec<Vec<String>>>>,
        update_global_search_stuff: *mut Action,
    ) -> Result<Self> {

        // Send the index back to the background thread, and wait until we get a response.
        sender_qt.send(Commands::DecodePackedFileDB).unwrap();
        sender_qt_data.send(Data::VecString(packed_file_path.borrow().to_vec())).unwrap();
        let packed_file_data = match check_message_validity_recv2(&receiver_qt) { 
            Data::DB(data) => Rc::new(RefCell::new(data)),
            Data::Error(error) => return Err(error),
            _ => panic!(THREADS_MESSAGE_ERROR), 
        };
        let table_definition = Rc::new(packed_file_data.borrow().table_definition.clone());

        // Create the dependency data for this table and populate it.
        let mut dependency_data: BTreeMap<i32, Vec<String>> = BTreeMap::new();
        for (index, field) in table_definition.fields.iter().enumerate() {
            if let Some(ref dependency) = field.field_is_reference {

                // Send the index back to the background thread, and wait until we get a response.
                sender_qt.send(Commands::DecodeDependencyDB).unwrap();
                sender_qt_data.send(Data::StringString(dependency.clone())).unwrap();
                match check_message_validity_recv2(&receiver_qt) { 
                    Data::VecString(data) => { dependency_data.insert(index as i32, data); },
                    Data::Error(_) => {},
                    _ => panic!(THREADS_MESSAGE_ERROR), 
                }
            }
        }

        let dependency_data = Rc::new(dependency_data);

        // Create the "Undo" history for the table.
        let history = Rc::new(RefCell::new(vec![]));
        let history_redo = Rc::new(RefCell::new(vec![]));
        let undo_lock = Rc::new(RefCell::new(false));
        let undo_model = StandardItemModel::new(()).into_raw();
        let undo_redo_enabler = Action::new(()).into_raw();

        // Create the TableView.
        let table_view = TableView::new().into_raw();
        let filter_model = SortFilterProxyModel::new().into_raw();
        let model = StandardItemModel::new(()).into_raw();

        // Make the last column fill all the available space, if the setting says so.
        if *SETTINGS.lock().unwrap().settings_bool.get("extend_last_column_on_tables").unwrap() { 
            unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }
        }

        // Create the filter's LineEdit.
        let row_filter_line_edit = LineEdit::new(()).into_raw();
        unsafe { row_filter_line_edit.as_mut().unwrap().set_placeholder_text(&QString::from_std_str("Type here to filter the rows in the table. Works with Regex too!")); }

        // Create the filter's column selector.
        let row_filter_column_selector = ComboBox::new().into_raw();
        let row_filter_column_list = StandardItemModel::new(()).into_raw();
        unsafe { row_filter_column_selector.as_mut().unwrap().set_model(row_filter_column_list as *mut AbstractItemModel); }
        for column in &table_definition.fields {
            let mut name = clean_column_names(&column.field_name);
            unsafe { row_filter_column_selector.as_mut().unwrap().add_item(&QString::from_std_str(&name)); }
        }

        // Create the filter's "Case Sensitive" button.
        let row_filter_case_sensitive_button = PushButton::new(&QString::from_std_str("Case Sensitive")).into_raw();
        unsafe { row_filter_case_sensitive_button.as_mut().unwrap().set_checkable(true); }

        // Prepare the TableView to have a Contextual Menu.
        unsafe { table_view.as_mut().unwrap().set_context_menu_policy(ContextMenuPolicy::Custom); }
        unsafe { table_view.as_mut().unwrap().set_horizontal_scroll_mode(ScrollMode::Pixel); }

        // Enable sorting the columns and make them movables.
        unsafe { table_view.as_mut().unwrap().set_sorting_enabled(true); }
        unsafe { table_view.as_mut().unwrap().sort_by_column((-1, SortOrder::Ascending)); }
        unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_sections_movable(true); }

        // Load the data to the Table. For some reason, if we do this after setting the titles of
        // the columns, the titles will be reseted to 1, 2, 3,... so we do this here.
        Self::load_data_to_table_view(&dependency_data, &packed_file_data.borrow(), table_view, model);

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
        for column in &table_definition.fields {
            column_selector.add_item(&QString::from_std_str(&clean_column_names(&column.field_name)));
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

        // Build the Column's "Data".
        build_columns(&table_definition, table_view, model);
        update_undo_model(model, undo_model);

        // Set both headers visible.
        unsafe { table_view.as_mut().unwrap().vertical_header().as_mut().unwrap().set_visible(true); }
        unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_visible(true); }

        // If we want to let the columns resize themselfs...
        if *SETTINGS.lock().unwrap().settings_bool.get("adjust_columns_to_content").unwrap() {
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
        let context_menu_apply_prefix_to_selection = context_menu_apply_submenu.add_action(&QString::from_std_str("&Apply Prefix to Selection"));

        let mut context_menu_clone_submenu = Menu::new(&QString::from_std_str("&Clone..."));
        let context_menu_clone = context_menu_clone_submenu.add_action(&QString::from_std_str("&Clone and Insert"));
        let context_menu_clone_and_append = context_menu_clone_submenu.add_action(&QString::from_std_str("Clone and &Append"));
        
        let mut context_menu_copy_submenu = Menu::new(&QString::from_std_str("&Copy..."));
        let context_menu_copy = context_menu_copy_submenu.add_action(&QString::from_std_str("&Copy"));
        let context_menu_copy_as_lua_table = context_menu_copy_submenu.add_action(&QString::from_std_str("&Copy as &LUA Table"));

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
        unsafe { context_menu_add.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("add_row").unwrap()))); }
        unsafe { context_menu_insert.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("insert_row").unwrap()))); }
        unsafe { context_menu_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("delete_row").unwrap()))); }
        unsafe { context_menu_apply_maths_to_selection.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("apply_maths_to_selection").unwrap()))); }
        unsafe { context_menu_apply_prefix_to_selection.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("apply_prefix_to_selection").unwrap()))); }
        unsafe { context_menu_clone.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("clone_row").unwrap()))); }
        unsafe { context_menu_clone_and_append.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("clone_and_append_row").unwrap()))); }
        unsafe { context_menu_copy.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("copy").unwrap()))); }
        unsafe { context_menu_copy_as_lua_table.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("copy_as_lua_table").unwrap()))); }
        unsafe { context_menu_paste_in_selection.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("paste_in_selection").unwrap()))); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("paste_as_new_row").unwrap()))); }
        unsafe { context_menu_paste_to_fill_selection.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("paste_to_fill_selection").unwrap()))); }
        unsafe { context_menu_search.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("search").unwrap()))); }
        unsafe { context_menu_import.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("import_tsv").unwrap()))); }
        unsafe { context_menu_export.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("export_tsv").unwrap()))); }
        unsafe { smart_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("smart_delete").unwrap()))); }
        unsafe { context_menu_undo.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("undo").unwrap()))); }
        unsafe { context_menu_redo.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().packed_files_db.get("redo").unwrap()))); }

        // Set the shortcuts to only trigger in the Table.
        unsafe { context_menu_add.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_insert.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_apply_maths_to_selection.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
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
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_apply_maths_to_selection); }
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
        unsafe { context_menu_apply_maths_to_selection.as_mut().unwrap().set_status_tip(&QString::from_std_str("Apply a simple mathematical operation to every cell in the selected cells.")); }
        unsafe { context_menu_apply_prefix_to_selection.as_mut().unwrap().set_status_tip(&QString::from_std_str("Apply a prefix to every cell in the selected cells.")); }
        unsafe { context_menu_clone.as_mut().unwrap().set_status_tip(&QString::from_std_str("Duplicate the selected rows and insert the new rows under the original ones.")); }
        unsafe { context_menu_clone_and_append.as_mut().unwrap().set_status_tip(&QString::from_std_str("Duplicate the selected rows and append the new rows at the end of the table.")); }
        unsafe { context_menu_copy.as_mut().unwrap().set_status_tip(&QString::from_std_str("Copy whatever is selected to the Clipboard.")); }
        unsafe { context_menu_copy_as_lua_table.as_mut().unwrap().set_status_tip(&QString::from_std_str("Turns the entire DB Table into a LUA Table and copies it to the clipboard.")); }
        unsafe { context_menu_paste_in_selection.as_mut().unwrap().set_status_tip(&QString::from_std_str("Try to paste whatever is in the Clipboard. Does nothing if the data is not compatible with the cell.")); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().set_status_tip(&QString::from_std_str("Try to paste whatever is in the Clipboard as new lines at the end of the table. Does nothing if the data is not compatible with the cell.")); }
        unsafe { context_menu_paste_to_fill_selection.as_mut().unwrap().set_status_tip(&QString::from_std_str("Try to paste whatever is in the Clipboard in EVERY CELL selected. Does nothing if the data is not compatible with the cell.")); }
        unsafe { context_menu_search.as_mut().unwrap().set_status_tip(&QString::from_std_str("Search what you want in the table. Also allows you to replace coincidences.")); }
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
        unsafe { context_menu.insert_separator(context_menu_undo); }
        unsafe { context_menu.insert_menu(context_menu_undo, context_menu_hide_show_submenu); }
        unsafe { context_menu.insert_separator(context_menu_undo); }

        // Create the "Hide/Show" slots and actions and connect them.
        let mut slots_hide_show_column = vec![];
        let mut actions_hide_show_column = vec![];
        for (index, column) in table_definition.fields.iter().enumerate() {

            let hide_show_slot = SlotBool::new(clone!(
                packed_file_path => move |state| {
                    unsafe { table_view.as_mut().unwrap().set_column_hidden(index as i32, state); }

                    // Update the state of the column in the table history.
                    if let Some(history_state) = HISTORY_TABLE_STATE.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
                        if state {
                            if !history_state.columns_state.hidden_columns.contains(&(index as i32)) {
                                history_state.columns_state.hidden_columns.push(index as i32);
                            }
                        }
                        else {
                            let pos = history_state.columns_state.hidden_columns.iter().position(|x| *x as usize == index).unwrap();
                            history_state.columns_state.hidden_columns.remove(pos);
                        }
                    }
                }
            ));

            let hide_show_action = unsafe { context_menu_hide_show_submenu.as_mut().unwrap().add_action(&QString::from_std_str(&clean_column_names(&column.field_name))) };
            unsafe { hide_show_action.as_mut().unwrap().set_checkable(true); }
            unsafe { hide_show_action.as_mut().unwrap().signals().toggled().connect(&hide_show_slot); }

            slots_hide_show_column.push(hide_show_slot);
            actions_hide_show_column.push(hide_show_action);
        }

        // Slots for the TableView...
        let slots = Self {
            slot_column_moved: SlotCIntCIntCInt::new(clone!(
                packed_file_path => move |_, visual_base, visual_new| {
                    if let Some(state) = HISTORY_TABLE_STATE.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
                        state.columns_state.visual_order.push((visual_base, visual_new));
                    }
                }
            )),

            slot_sort_order_column_changed: SlotCIntQtCoreQtSortOrder::new(clone!(
                packed_file_path => move |column, order| {
                    if let Some(state) = HISTORY_TABLE_STATE.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
                        state.columns_state.sorting_column = (column, if let SortOrder::Ascending = order { false } else { true });
                    }
                }
            )),

            slot_undo: SlotNoArgs::new(clone!(
                global_search_explicit_paths,
                dependency_data,
                packed_file_path,
                app_ui,
                is_modified,
                packed_file_data,
                sender_qt,
                sender_qt_data,
                undo_lock,
                history_redo,
                history => move || {

                    Self::undo_redo(
                        &app_ui,
                        &dependency_data,
                        &sender_qt,
                        &sender_qt_data,
                        &is_modified,
                        &packed_file_path,
                        &packed_file_data,
                        table_view,
                        model,
                        filter_model,
                        &history,
                        &history_redo,
                        &global_search_explicit_paths,
                        update_global_search_stuff,
                        &undo_lock,
                    );

                    update_undo_model(model, undo_model);
                    unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                    unsafe { update_search_stuff.as_mut().unwrap().trigger(); }
                }
            )),

            slot_redo: SlotNoArgs::new(clone!(
                global_search_explicit_paths,
                dependency_data,
                packed_file_path,
                app_ui,
                is_modified,
                packed_file_data,
                sender_qt,
                sender_qt_data,
                undo_lock,
                history_redo,
                history => move || {

                    Self::undo_redo(
                        &app_ui,
                        &dependency_data,
                        &sender_qt,
                        &sender_qt_data,
                        &is_modified,
                        &packed_file_path,
                        &packed_file_data,
                        table_view,
                        model,
                        filter_model,
                        &history_redo,
                        &history,
                        &global_search_explicit_paths,
                        update_global_search_stuff,
                        &undo_lock,
                    );

                    update_undo_model(model, undo_model);
                    unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                    unsafe { update_search_stuff.as_mut().unwrap().trigger(); }
                }
            )),

            slot_undo_redo_enabler: SlotNoArgs::new(clone!(
                app_ui,
                history_redo,
                packed_file_path,
                history => move || { 
                    unsafe {
                        if history.borrow().is_empty() { 
                            context_menu_undo.as_mut().unwrap().set_enabled(false); 
                            undo_paint_for_packed_file(&app_ui, model, &packed_file_path);
                        }
                        else { context_menu_undo.as_mut().unwrap().set_enabled(true); }
                        
                        if history_redo.borrow().is_empty() { context_menu_redo.as_mut().unwrap().set_enabled(false); }
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
                                if *field_type == FieldType::Integer || *field_type == FieldType::LongInteger || *field_type == FieldType::Float { continue }
                                else { can_apply = false; break } 
                            }
                            context_menu_apply_maths_to_selection.as_mut().unwrap().set_enabled(can_apply);

                            let mut can_apply = true;
                            for column in &columns {
                                let field_type = &table_definition.fields[*column as usize].field_type;
                                if *field_type == FieldType::StringU8 || *field_type == FieldType::StringU16 || *field_type == FieldType::OptionalStringU8 || *field_type == FieldType::OptionalStringU16 { continue }
                                else { can_apply = false; break } 
                            }
                            context_menu_apply_prefix_to_selection.as_mut().unwrap().set_enabled(can_apply);
                        }
                    }

                    // Otherwise, disable them.
                    else {
                        unsafe {
                            context_menu_apply_maths_to_selection.as_mut().unwrap().set_enabled(false);
                            context_menu_apply_prefix_to_selection.as_mut().unwrap().set_enabled(false);
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
                is_modified,
                packed_file_data,
                sender_qt,
                sender_qt_data => move |_,_,roles| {

                    // To avoid doing this multiple times due to the cell painting stuff, we need to check the role.
                    // This has to be allowed ONLY if the role is 0 (DisplayText), 2 (EditorText) or 10 (CheckStateRole).
                    // 16 is a role we use as an special trigger for this.
                    if roles.contains(&0) || roles.contains(&2) || roles.contains(&10) || roles.contains(&16) {

                        // Try to save the PackedFile to the main PackFile. If it fails, revert the last change, 
                        // In theory, only a manual bad edit or a replace current/all of a number column are the only things that can cause this to fail. 
                        if let Err(error) = Self::save_to_packed_file(
                            &sender_qt,
                            &sender_qt_data,
                            &is_modified,
                            &app_ui,
                            &packed_file_data,
                            &packed_file_path,
                            model,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
                        ) { show_dialog(app_ui.window, false, error); }

                        // Otherwise, update the needed stuff.
                        unsafe { update_search_stuff.as_mut().unwrap().trigger(); }
                    }
                }
            )),

            slot_item_changed: SlotStandardItemMutPtr::new(clone!(
                history,
                history_redo,
                dependency_data,
                table_definition => move |item| {

                    // If we are NOT UNDOING, paint the item as edited and add the edition to the undo list.
                    if !*undo_lock.borrow() {
                        let item_old = unsafe { &*undo_model.as_mut().unwrap().item((item.as_mut().unwrap().row(), item.as_mut().unwrap().column())) };
                        unsafe { history.borrow_mut().push(TableOperations::Editing(vec![((item.as_mut().unwrap().row(), item.as_mut().unwrap().column()), item_old.clone()); 1])); }
                        history_redo.borrow_mut().clear();

                        // We block the saving for painting, so this doesn't get rettriggered again.
                        let mut blocker = unsafe { SignalBlocker::new(model.as_mut().unwrap().static_cast_mut() as &mut Object) };
                        unsafe { item.as_mut().unwrap().set_background(&Brush::new(if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkYellow } else { GlobalColor::Yellow })); }
                        blocker.unblock();

                        update_undo_model(model, undo_model);
                        unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }

                    }

                    // If we have the dependency stuff enabled, check if it's a valid reference.
                    if *SETTINGS.lock().unwrap().settings_bool.get("use_dependency_checker").unwrap() {
                        let column = unsafe { item.as_mut().unwrap().column() };
                        if table_definition.fields[column as usize].field_is_reference.is_some() {
                            check_references(&dependency_data, column, item);
                        }
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
                history,
                history_redo,
                table_definition => move |_| {

                    // Create a new list of StandardItem.
                    let mut qlist = ListStandardItemMutPtr::new(());

                    // For each field in the definition...
                    for field in &table_definition.fields {

                        // Create a new Item.
                        let mut item = match field.field_type {

                            // This one needs a couple of changes before turning it into an item in the table.
                            FieldType::Boolean => {
                                let mut item = StandardItem::new(());
                                item.set_editable(false);
                                item.set_checkable(true);
                                item.set_check_state(CheckState::Checked);
                                item
                            },
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

                        // Paint the cells.
                        item.set_background(&Brush::new(if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkGreen } else { GlobalColor::Green }));

                        // Add the item to the list.
                        unsafe { qlist.append_unsafe(&item.into_raw()); }
                    }

                    // Append the new row.
                    unsafe { model.as_mut().unwrap().append_row(&qlist); }

                    // Add the operation to the undo history.
                    unsafe { history.borrow_mut().push(TableOperations::AddRows(vec![model.as_mut().unwrap().row_count(()) - 1; 1])); }
                    history_redo.borrow_mut().clear();
                    update_undo_model(model, undo_model);
                    unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                }
            )),
            slot_context_menu_insert: SlotBool::new(clone!(
                history,
                history_redo,
                table_definition => move |_| {

                    // Create a new list of StandardItem.
                    let mut qlist = ListStandardItemMutPtr::new(());

                    // For each field in the definition...
                    for field in &table_definition.fields {

                        // Create a new Item.
                        let mut item = match field.field_type {

                            // This one needs a couple of changes before turning it into an item in the table.
                            FieldType::Boolean => {
                                let mut item = StandardItem::new(());
                                item.set_editable(false);
                                item.set_checkable(true);
                                item.set_check_state(CheckState::Checked);
                                item
                            },
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

                        // Paint the cells.
                        item.set_background(&Brush::new(if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkGreen } else { GlobalColor::Green }));

                        // Add the item to the list.
                        unsafe { qlist.append_unsafe(&item.into_raw()); }
                    }

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

                    history.borrow_mut().push(TableOperations::AddRows(vec![row; 1]));
                    history_redo.borrow_mut().clear();
                    update_undo_model(model, undo_model);
                    unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                }
            )),
            slot_context_menu_delete: SlotBool::new(clone!(
                global_search_explicit_paths,
                packed_file_path,
                app_ui,
                is_modified,
                packed_file_data,
                history,
                history_redo,
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

                    // Delete every selected row.
                    let mut rows_data = vec![];
                    unsafe { rows.iter().for_each(|x| rows_data.push(model.as_mut().unwrap().take_row(*x))); }

                    // If we deleted something, try to save the PackedFile to the main PackFile. This cannot fail here, so we can ignore the result
                    if !rows.is_empty() {
                        let _y = Self::save_to_packed_file(
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

                        history.borrow_mut().push(TableOperations::RemoveRows((rows, rows_data)));
                        history_redo.borrow_mut().clear();
                        update_undo_model(model, undo_model);
                        unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                    }
                }
            )),

            slot_context_menu_apply_maths_to_selection: SlotBool::new(clone!(
                history,
                history_redo,
                table_definition,
                app_ui => move |_| {

                    // If we got an operation and a value, get all the cells in the selection, try to apply the operation to them and,
                    // if the resulting value is valid in all of them, apply it.
                    if let Some((mut operation, value)) = create_apply_maths_dialog(&app_ui) {

                        // For some reason Qt adds & to the X operation, so we need to delete it.
                        if let Some(index) = operation.find('&') { operation.remove(index); }

                        let mut results = vec![];
                        let indexes = unsafe { filter_model.as_mut().unwrap().map_selection_to_source(&table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };
                        for index in 0..indexes.count(()) {
                            let model_index = indexes.at(index);
                            if model_index.is_valid() { 

                                // For Integers and Long Integers is easy. Just try to do the operation and return an error if it fails.
                                let field_type = &table_definition.fields[model_index.column() as usize].field_type;
                                if *field_type == FieldType::LongInteger || *field_type == FieldType::Integer {
                                    let value_to_operate = unsafe { model.as_ref().unwrap().item_from_index(model_index).as_ref().unwrap().data(2).to_long_long() };

                                    let result = match &*operation {
                                        "+" => value_to_operate.checked_add(value.round() as i64),
                                        "-" => value_to_operate.checked_sub(value.round() as i64),
                                        "*" => value_to_operate.checked_mul(value.round() as i64),
                                        "/" => value_to_operate.checked_div(value.round() as i64),
                                        _ => unreachable!()
                                    };

                                    let result = match result {
                                        Some(result) => result,
                                        None => return show_dialog(app_ui.window, false, ErrorKind::DBTableApplyMathsOverflow)
                                    };

                                    // If the value hasn't change (like when we add 0), set the operation as "None".
                                    if result == value_to_operate { results.push(MathOperationResult::None) }

                                    // For i32, we need to make sure the value is also a valid i32.
                                    else if *field_type == FieldType::Integer {
                                        if let Ok(result) = format!("{}", result).parse::<i32>() { results.push(MathOperationResult::Integer(result)) }
                                        else { return show_dialog(app_ui.window, false, ErrorKind::DBTableApplyMathsOverflow) }
                                    } else { results.push(MathOperationResult::LongInteger(result)) }         
                                }

                                // There is not a built-in method to check for overflow with floats, so we have to do some thinking.
                                // Yo, maths! Ere I go again!
                                else if *field_type == FieldType::Float {
                                    let value_to_operate = unsafe { model.as_ref().unwrap().item_from_index(model_index).as_ref().unwrap().data(2).to_double() };

                                    let result = match &*operation {
                                        "+" => value_to_operate + value,
                                        "-" => value_to_operate - value,
                                        "*" => value_to_operate * value,
                                        "/" => if value != 0f64 { value_to_operate / value } else { return show_dialog(app_ui.window, false, ErrorKind::ThereIsAnSpecialPlaceInHellForYou) },
                                        _ => unreachable!()
                                    };

                                    // If the value hasn't change (like when we add 0), set the operation as "None".
                                    if result == value_to_operate { results.push(MathOperationResult::None) }

                                    else if let Ok(result) = format!("{}", result).parse::<f32>() { results.push(MathOperationResult::Float(result)) }
                                    else { return show_dialog(app_ui.window, false, ErrorKind::DBTableApplyMathsOverflow) }
                                }
                            }
                        }

                        // If every result is a valid result, then iterate again over every cell applying the new value.
                        for index in 0..indexes.count(()) {
                            let model_index = indexes.at(index);
                            match results[index as usize] {
                                MathOperationResult::Integer(x) => unsafe { model.as_mut().unwrap().item_from_index(model_index).as_mut().unwrap().set_data((&Variant::new0(x), 2)) },
                                MathOperationResult::LongInteger(x) => unsafe { model.as_mut().unwrap().item_from_index(model_index).as_mut().unwrap().set_data((&Variant::new2(x), 2)) },
                                MathOperationResult::Float(x) => unsafe { model.as_mut().unwrap().item_from_index(model_index).as_mut().unwrap().set_data((&Variant::new2(x), 2)) }
                                MathOperationResult::None => continue,
                            }
                        }

                        // If we finished doing maths, fix the undo history to have all the previous changes merged into one.
                        // Keep in mind that `None` results should be ignored here.
                        let valid_results = results.iter().filter(|x| **x != MathOperationResult::None).count();

                        if valid_results > 0 {
                            let len = history.borrow().len();
                            let mut edits_data = vec![];
                            {
                                let mut history = history.borrow_mut();
                                let mut edits = history.drain((len - valid_results)..);
                                for edit in &mut edits { if let TableOperations::Editing(mut edit) = edit { edits_data.append(&mut edit); }}
                            }

                            history.borrow_mut().push(TableOperations::Editing(edits_data));
                            history_redo.borrow_mut().clear();
                            update_undo_model(model, undo_model);
                            unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                        }
                    }
                }
            )),

            slot_context_menu_apply_prefix_to_selection: SlotBool::new(clone!(
                history,
                history_redo,
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

                        // If we finished appling prefixes, fix the undo history to have all the previous changes merged into one.
                        // Keep in mind that `None` results should be ignored here.
                        let len = history.borrow().len();
                        let mut edits_data = vec![];
                        {
                            let mut history = history.borrow_mut();
                            let mut edits = history.drain((len - results.len())..);
                            for edit in &mut edits { if let TableOperations::Editing(mut edit) = edit { edits_data.append(&mut edit); }}
                        }

                        history.borrow_mut().push(TableOperations::Editing(edits_data));
                        history_redo.borrow_mut().clear();
                        update_undo_model(model, undo_model);
                        unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                    }
                }
            )),

            slot_context_menu_clone: SlotBool::new(clone!(
                global_search_explicit_paths,
                packed_file_path,
                app_ui,
                is_modified,
                table_definition,
                packed_file_data,
                history,
                history_redo,
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
                            unsafe { item.as_mut().unwrap().set_background(&Brush::new(if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkGreen } else { GlobalColor::Green })); }
                            unsafe { qlist.append_unsafe(&item); }
                        }

                        // Insert the new row after the original one.
                        unsafe { model.as_mut().unwrap().insert_row((row + 1, &qlist)); }
                    }

                    // If we cloned something, try to save the PackedFile to the main PackFile.
                    if !rows.is_empty() {
                        let _y = Self::save_to_packed_file(
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
                        rows.iter_mut().for_each(|x| *x += 1);
                        history.borrow_mut().push(TableOperations::AddRows(rows));
                        history_redo.borrow_mut().clear();
                        update_undo_model(model, undo_model);
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
                table_definition,
                history,
                history_redo,
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
                            unsafe { item.as_mut().unwrap().set_background(&Brush::new(if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkGreen } else { GlobalColor::Green })); }
                            unsafe { qlist.append_unsafe(&item); }
                        }

                        // Insert the new row after the original one.
                        unsafe { model.as_mut().unwrap().append_row(&qlist); }
                    }

                    // If we cloned something, try to save the PackedFile to the main PackFile.
                    if !rows.is_empty() {
                        let _y = Self::save_to_packed_file(
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
                        let range = (total_rows - rows.len() as i32..total_rows).collect::<Vec<i32>>();
                        history.borrow_mut().push(TableOperations::AddRows(range));
                        history_redo.borrow_mut().clear();
                        update_undo_model(model, undo_model);
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
                        if header.visual_index(a.column()) < header.visual_index(b.column()) { return Ordering::Less }
                        else { return Ordering::Greater }
                    } else { return Ordering::Equal }
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
                table_definition,
                packed_file_data => move |_| {

                    // Get all the rows into a Vec<Vec<String>>, so we can deal with them more easely.
                    let mut entries = vec![];
                    for row in &packed_file_data.borrow().entries {
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
                                        if decimals > &3 { format!("{}", format!("{:.3}", data).parse::<f32>().unwrap()) }
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
                                DecodedData::OptionalStringU16(ref data) => format!("\"{}\"", data.replace('\\', "\\\\")),
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
            slot_context_menu_paste_in_selection: SlotBool::new(clone!(
                history,
                history_redo,
                table_definition => move |_| {

                    // If whatever it's in the Clipboard is pasteable in our selection...
                    if check_clipboard(&table_definition, table_view, model, filter_model) {

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
                                if header.visual_index(a.column()) < header.visual_index(b.column()) { return Ordering::Less }
                                else { return Ordering::Greater }
                            } else { return Ordering::Equal }
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
                            let column = unsafe { cell.0.as_mut().unwrap().index().column() };
                            match table_definition.fields[column as usize].field_type {

                                FieldType::Boolean => {
                                    let current_value = unsafe { cell.0.as_mut().unwrap().check_state() };
                                    let new_value = if cell.1.to_lowercase() == "true" || cell.1 == "1" { CheckState::Checked } else { CheckState::Unchecked };
                                    if current_value != new_value { 
                                        unsafe { cell.0.as_mut().unwrap().set_check_state(new_value); }
                                        changed_cells += 1;
                                    }
                                },

                                FieldType::Float => {
                                    let current_value = unsafe { cell.0.as_mut().unwrap().text().to_std_string() };
                                    if &*current_value != cell.1 {
                                        unsafe { cell.0.as_mut().unwrap().set_data((&Variant::new2(cell.1.parse::<f32>().unwrap()), 2)); }
                                        changed_cells += 1;
                                    }
                                },

                                FieldType::Integer => {
                                    let current_value = unsafe { cell.0.as_mut().unwrap().text().to_std_string() };
                                    if &*current_value != cell.1 {
                                        unsafe { cell.0.as_mut().unwrap().set_data((&Variant::new0(cell.1.parse::<i32>().unwrap()), 2)); }
                                        changed_cells += 1;
                                    }
                                },

                                FieldType::LongInteger => {
                                    let current_value = unsafe { cell.0.as_mut().unwrap().text().to_std_string() };
                                    if &*current_value != cell.1 {
                                        unsafe { cell.0.as_mut().unwrap().set_data((&Variant::new2(cell.1.parse::<i64>().unwrap()), 2)); }
                                        changed_cells += 1;
                                    }
                                },

                                _ => {
                                    let current_value = unsafe { cell.0.as_mut().unwrap().text().to_std_string() };
                                    if &*current_value != cell.1 {
                                        unsafe { cell.0.as_mut().unwrap().set_text(&QString::from_std_str(cell.1)); }
                                        changed_cells += 1;
                                    }
                                }
                            }
                        }

                        // Fix the undo history to have all the previous changed merged into one.
                        if changed_cells > 0 {
                            let len = history.borrow().len();
                            let mut edits_data = vec![];
                            {
                                let mut history = history.borrow_mut();
                                let mut edits = history.drain((len - changed_cells)..);
                                for edit in &mut edits { if let TableOperations::Editing(mut edit) = edit { edits_data.append(&mut edit); }}
                            }

                            history.borrow_mut().push(TableOperations::Editing(edits_data));
                            history_redo.borrow_mut().clear();
                            update_undo_model(model, undo_model);
                            unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }                           
                        }
                    }
                }
            )),

            slot_context_menu_paste_as_new_lines: SlotBool::new(clone!(
                global_search_explicit_paths,
                dependency_data,
                packed_file_path,
                app_ui,
                is_modified,
                table_definition,
                packed_file_data,
                history,
                history_redo,
                sender_qt,
                sender_qt_data => move |_| {

                    // If whatever it's in the Clipboard is pasteable in our selection...
                    if check_clipboard_append_rows(table_view, &table_definition) {

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
                                    item.set_background(&Brush::new(if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                },
                                
                                FieldType::Float => {
                                    item.set_data((&Variant::new2(cell.parse::<f32>().unwrap()), 2));
                                    item.set_background(&Brush::new(if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                },

                                FieldType::Integer => {
                                    item.set_data((&Variant::new0(cell.parse::<i32>().unwrap()), 2));
                                    item.set_background(&Brush::new(if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                },

                                FieldType::LongInteger => {
                                    item.set_data((&Variant::new2(cell.parse::<i64>().unwrap()), 2));
                                    item.set_background(&Brush::new(if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                },

                                // In any other case, we treat it as a string. Type-checking is done before this and while saving.
                                _ => {
                                    item.set_text(&QString::from_std_str(cell));
                                    item.set_background(&Brush::new(if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                }
                            }

                            // If we have the dependency stuff enabled, check if it's a valid reference.
                            if *SETTINGS.lock().unwrap().settings_bool.get("use_dependency_checker").unwrap() {
                                if field.field_is_reference.is_some() {
                                    check_references(&dependency_data, column as i32, item.as_mut_ptr());
                                }
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

                                item.set_background(&Brush::new(if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkGreen } else { GlobalColor::Green }));
                                qlist_unordered.push((column_logical_index, item.into_raw()));
                            }

                            // Append the list to the Table.
                            qlist_unordered.sort_unstable_by_key(|x| x.0);
                            for (_, item) in &qlist_unordered { unsafe { qlist.append_unsafe(&item.clone()); }}
                            unsafe { model.as_mut().unwrap().append_row(&qlist); }
                        }

                        // If we pasted something, try to save the PackedFile to the main PackFile.
                        if !text.is_empty() {
                            let _y = Self::save_to_packed_file(
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
                            let mut rows = vec![];
                            unsafe { (undo_model.as_mut().unwrap().row_count(())..model.as_mut().unwrap().row_count(())).for_each(|x| rows.push(x)); }
                            history.borrow_mut().push(TableOperations::AddRows(rows));
                            history_redo.borrow_mut().clear();
                            update_undo_model(model, undo_model);
                            unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
                        }
                    }
                }
            )),

            slot_context_menu_paste_to_fill_selection: SlotBool::new(clone!(
                history,
                history_redo,
                table_definition => move |_| {

                    // If whatever it's in the Clipboard is pasteable in our selection...
                    if check_clipboard_to_fill_selection(&table_definition, table_view, model, filter_model) {

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
                                        if &*current_value != text {
                                            unsafe { item.as_mut().unwrap().set_data((&Variant::new2(text.parse::<f32>().unwrap()), 2)); }
                                            changed_cells += 1;
                                        }
                                    },

                                    FieldType::Integer => {
                                        let current_value = unsafe { item.as_mut().unwrap().text().to_std_string() };
                                        if &*current_value != text {
                                            unsafe { item.as_mut().unwrap().set_data((&Variant::new0(text.parse::<i32>().unwrap()), 2)); }
                                            changed_cells += 1;
                                        }
                                    },

                                    FieldType::LongInteger => {
                                        let current_value = unsafe { item.as_mut().unwrap().text().to_std_string() };
                                        if &*current_value != text {
                                            unsafe { item.as_mut().unwrap().set_data((&Variant::new2(text.parse::<i64>().unwrap()), 2)); }
                                            changed_cells += 1;
                                        }
                                    },

                                    _ => {
                                        let current_value = unsafe { item.as_mut().unwrap().text().to_std_string() };
                                        if &*current_value != text {
                                            unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str(&text)); }
                                            changed_cells += 1;
                                        }
                                    }
                                }
                            }
                        }

                        // Fix the undo history to have all the previous changed merged into one.
                        if changed_cells > 0 {
                            let len = history.borrow().len();
                            let mut edits_data = vec![];
                            {
                                let mut history = history.borrow_mut();
                                let mut edits = history.drain((len - changed_cells)..);
                                for edit in &mut edits { if let TableOperations::Editing(mut edit) = edit { edits_data.append(&mut edit); }}
                            }

                            history.borrow_mut().push(TableOperations::Editing(edits_data));
                            history_redo.borrow_mut().clear();
                            update_undo_model(model, undo_model);
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
                table_definition,
                packed_file_data,
                packed_file_path,
                history,
                history_redo,
                sender_qt,
                sender_qt_data,
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
                        sender_qt.send(Commands::ImportTSVPackedFileDB).unwrap();
                        sender_qt_data.send(Data::DBPathBuf((packed_file_data.borrow().clone(), path))).unwrap();

                        match check_message_validity_recv2(&receiver_qt) {
                            Data::DB(new_db_data) => Self::load_data_to_table_view(&dependency_data, &new_db_data, table_view, model),
                            Data::Error(error) => return show_dialog(app_ui.window, false, error),
                            _ => panic!(THREADS_MESSAGE_ERROR),
                        }

                        // Build the Column's "Data".
                        build_columns(&table_definition, table_view, model);

                        if *SETTINGS.lock().unwrap().settings_bool.get("adjust_columns_to_content").unwrap() {
                            unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
                        }

                        // Make a copy of the old data for the undo system.
                        let old_data = packed_file_data.borrow().clone();

                        // Try to save the PackedFile to the main PackFile.
                        let _y = Self::save_to_packed_file(
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
                        history.borrow_mut().push(TableOperations::ImportTSVDB(old_data));
                        history_redo.borrow_mut().clear();
                        update_undo_model(model, undo_model);
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

                    // Run it and, if we receive 1 (Accept), export the DB Table.
                    if file_dialog.exec() == 1 {

                        let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());
                        sender_qt.send(Commands::ExportTSVPackedFileDB).unwrap();
                        sender_qt_data.send(Data::DBPathBuf((packed_file_data.borrow().clone(), path))).unwrap();

                        // If there is an error, report it. Otherwise, we're done.
                        match check_message_validity_recv2(&receiver_qt) {
                            Data::Success => return (),
                            Data::Error(error) => return show_dialog(app_ui.window, false, error),
                            _ => panic!(THREADS_MESSAGE_ERROR),
                        }
                    }
                }
            )),
            slot_smart_delete: SlotBool::new(clone!(
                global_search_explicit_paths,
                app_ui,
                is_modified,
                table_definition,
                packed_file_data,
                packed_file_path,
                history,
                history_redo,
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

                    let mut edits = vec![];
                    let mut removed_rows = vec![];
                    for (key, values) in cells.iter().rev() {

                        // If we have selected all the cells from a row, delete the row.
                        if values.len() == table_definition.fields.len() { unsafe { removed_rows.push((*key, model.as_mut().unwrap().take_row(*key))); }}

                        // Otherwise, delete the values on the cells, not the cells themselfs.
                        else { 
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

                    // When you delete a row, the save has to be triggered manually. For cell edits it get's triggered automatically.
                    if !cells.is_empty() {
                        let _y = Self::save_to_packed_file(
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

                        // Update the undo stuff. This is a bit special, as we have to remove all the automatically created "Editing" undos.
                        let len = history.borrow().len();
                        history.borrow_mut().truncate(len - edits.len());
                        history.borrow_mut().push(TableOperations::SmartDelete((edits, removed_rows)));
                        history_redo.borrow_mut().clear();
                        update_undo_model(model, undo_model);
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
                                let mut matches_unprocessed = unsafe { model.as_mut().unwrap().find_items((&QString::from_std_str(text), flags.clone(), index as i32)) };
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
                            let mut matches_unprocessed = unsafe { model.as_mut().unwrap().find_items((&QString::from_std_str(text), flags.clone(), column)) };
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
                    if let Some(state) = HISTORY_TABLE_STATE.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
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
                                let mut matches_unprocessed = unsafe { model.as_mut().unwrap().find_items((&text, flags.clone(), index as i32)) };
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
                            let mut matches_unprocessed = unsafe { model.as_mut().unwrap().find_items((&text, flags.clone(), column as i32)) };
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
                    if let Some(state) = HISTORY_TABLE_STATE.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
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

                            match table_definition.fields[unsafe { item.as_mut().unwrap().column() as usize }].field_type {
                                FieldType::Float => unsafe { item.as_mut().unwrap().set_data((&Variant::new2(data.1.parse::<f32>().unwrap()), 2)); }
                                FieldType::Integer => unsafe { item.as_mut().unwrap().set_data((&Variant::new0(data.1.parse::<i32>().unwrap()), 2)); }
                                FieldType::LongInteger => unsafe { item.as_mut().unwrap().set_data((&Variant::new2(data.1.parse::<i64>().unwrap()), 2)); }
                                _ => unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str(&data.1)); }
                            }

                            // If we finished replacing, fix the undo history to have all the previous changed merged into one.
                            if index == positions_and_texts.len() - 1 {
                                let len = history.borrow().len();
                                let mut edits_data = vec![];
                                {
                                    let mut history = history.borrow_mut();
                                    let mut edits = history.drain((len - positions_and_texts.len())..);
                                    for edit in &mut edits { if let TableOperations::Editing(mut edit) = edit { edits_data.append(&mut edit); }}
                                }

                                history.borrow_mut().push(TableOperations::Editing(edits_data));
                                history_redo.borrow_mut().clear();
                                update_undo_model(model, undo_model);
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
        unsafe { context_menu_apply_maths_to_selection.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_apply_maths_to_selection); }
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
            context_menu_apply_maths_to_selection.as_mut().unwrap().set_enabled(false);
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
            context_menu_undo.as_mut().unwrap().set_enabled(false);
            context_menu_redo.as_mut().unwrap().set_enabled(false);
        }

        // Trigger the "Enable/Disable" slot every time we change the selection in the TreeView.
        unsafe { table_view.as_mut().unwrap().selection_model().as_ref().unwrap().signals().selection_changed().connect(&slots.slot_context_menu_enabler); }

        // If we got an entry for this PackedFile in the state's history, use it.
        if HISTORY_TABLE_STATE.lock().unwrap().get(&*packed_file_path.borrow()).is_some() {
            if let Some(state_data) = HISTORY_TABLE_STATE.lock().unwrap().get_mut(&*packed_file_path.borrow()) {

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
                
                if *SETTINGS.lock().unwrap().settings_bool.get("remember_column_state").unwrap() {
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
        else { HISTORY_TABLE_STATE.lock().unwrap().insert(packed_file_path.borrow().to_vec(), TableState::new_empty()); }

        // Retrigger the filter, so the table get's updated properly.
        unsafe { row_filter_case_sensitive_button.as_mut().unwrap().set_checked(!row_filter_case_sensitive_button.as_mut().unwrap().is_checked()); }
        unsafe { row_filter_case_sensitive_button.as_mut().unwrap().set_checked(!row_filter_case_sensitive_button.as_mut().unwrap().is_checked()); }

        // Return the slots to keep them as hostages.
        return Ok(slots)
    }

    /// This function loads the data from a LocData into a TableView.
    pub fn load_data_to_table_view(
        dependency_data: &BTreeMap<i32, Vec<String>>,
        packed_file_data: &DB,
        table_view: *mut TableView,
        model: *mut StandardItemModel,
    ) {
        // First, we delete all the data from the `ListStore`. Just in case there is something there.
        // This wipes out header information, so remember to run "build_columns" after this.
        unsafe { model.as_mut().unwrap().clear(); }

        for entry in &packed_file_data.entries {
            let mut qlist = ListStandardItemMutPtr::new(());
            for (index, field) in entry.iter().enumerate() {

                // Create a new Item.
                let mut item = match *field {

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
                                if decimals > &3 { format!("{:.3}", data).parse::<f32>().unwrap() }
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
                if *SETTINGS.lock().unwrap().settings_bool.get("use_dependency_checker").unwrap() {
                    if packed_file_data.table_definition.fields[index].field_is_reference.is_some() {
                        check_references(dependency_data, index as i32, item.as_mut_ptr());
                    }
                }
                unsafe { qlist.append_unsafe(&item.into_raw()); }
            }
            unsafe { model.as_mut().unwrap().append_row(&qlist); }
        }

        // If the table it's empty, we add an empty row and delete it, so the "columns" get created.
        if packed_file_data.entries.len() == 0 {
            let mut qlist = ListStandardItemMutPtr::new(());
            for field in &packed_file_data.table_definition.fields {
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
        for (column, field) in packed_file_data.table_definition.fields.iter().enumerate() {
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
        for (column, data) in dependency_data {
            let mut list = StringList::new(());
            data.iter().for_each(|x| list.append(&QString::from_std_str(x)));
            let list: *mut StringList = &mut list;
            unsafe { qt_custom_stuff::new_combobox_item_delegate(table_view as *mut Object, *column, list as *const StringList, true)};
        }
    }

    /// This function returns a DBData with all the stuff in the table. This can and will fail in case
    /// the data of a field cannot be parsed to the type of that field. Beware of that.
    pub fn return_data_from_table_view(
        packed_file_data: &mut DB,
        model: *mut StandardItemModel,
    ) -> Result<()> {

        // This list is to store the new data before passing it to the PackedFile, just in case it fails.
        let mut new_data: Vec<Vec<DecodedData>> = vec![];

        // Get the data from the table, row by row, ensuring their type is correct.
        let rows = unsafe { model.as_mut().unwrap().row_count(()) };
        for row in 0..rows {
            let mut new_row: Vec<DecodedData> = vec![];
            for (column, field) in packed_file_data.table_definition.fields.iter().enumerate() {

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
            new_data.push(new_row);
        }

        // If we reached this place, it means there has been no errors while parsing the data, so we
        // replace the old entries with the new ones.
        packed_file_data.entries = new_data;

        // Return success.
        Ok(())
    }

    /// Function to save the data from the current StandardItemModel to the PackFile.
    pub fn save_to_packed_file(
        sender_qt: &Sender<Commands>,
        sender_qt_data: &Sender<Data>,
        is_modified: &Rc<RefCell<bool>>,
        app_ui: &AppUI,
        data: &Rc<RefCell<DB>>,
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        model: *mut StandardItemModel,
        global_search_explicit_paths: &Rc<RefCell<Vec<Vec<String>>>>,
        update_global_search_stuff: *mut Action,
    ) -> Result<()> {

        // Update the DB with the data in the table, or report error if it fails.
        Self::return_data_from_table_view(&mut data.borrow_mut(), model)?;

        // Tell the background thread to start saving the PackedFile.
        sender_qt.send(Commands::EncodePackedFileDB).unwrap();
        sender_qt_data.send(Data::DBVecString((data.borrow().clone(), packed_file_path.borrow().to_vec()))).unwrap();

        // Set the mod as "Modified".
        *is_modified.borrow_mut() = set_modified(true, &app_ui, Some(packed_file_path.borrow().to_vec()));

        // Update the global search stuff, if needed.
        global_search_explicit_paths.borrow_mut().push(packed_file_path.borrow().to_vec());
        unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }

        // If the table has been saved without problems, return success.
        Ok(())
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
        is_modified: &Rc<RefCell<bool>>,
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        data: &Rc<RefCell<DB>>,
        table_view: *mut TableView,
        model: *mut StandardItemModel,
        filter_model: *mut SortFilterProxyModel,
        history_source: &Rc<RefCell<Vec<TableOperations>>>, 
        history_opposite: &Rc<RefCell<Vec<TableOperations>>>,
        global_search_explicit_paths: &Rc<RefCell<Vec<Vec<String>>>>,
        update_global_search_stuff: *mut Action,
        undo_lock: &Rc<RefCell<bool>>,
    ) {

        // Get the last operation in the Undo History, or return if there is none.
        let operation = if let Some(operation) = history_source.borrow_mut().pop() { operation } else { return };
        match operation {
            TableOperations::Editing(editions) => {

                // Prepare the redo operation, then do the rest.
                let mut redo_editions = vec![];
                unsafe { editions.iter().for_each(|x| redo_editions.push((((x.0).0, (x.0).1), (&*model.as_mut().unwrap().item(((x.0).0, (x.0).1))).clone()))); }
                history_opposite.borrow_mut().push(TableOperations::Editing(redo_editions));
    
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
            TableOperations::AddRows(rows) => {

                // Prepare the redo operation. Rows are removed from top to bottom, so we have to receive them sorted from bottom to top (9->1).
                let mut new_rows = vec![];
                unsafe { rows.iter().rev().for_each(|x| new_rows.push(model.as_mut().unwrap().take_row(*x))); }
                history_opposite.borrow_mut().push(TableOperations::RemoveRows((rows.to_vec(), new_rows)));

                let _y = Self::save_to_packed_file(
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

            TableOperations::RemoveRows((rows, rows_data)) => {

                // Prepare the redo operation and insert the removed rows. Then select the new rows.
                unsafe { rows.iter().enumerate().rev().for_each(|(index, x)| model.as_mut().unwrap().insert_row((*x, &rows_data[index]))); }
                history_opposite.borrow_mut().push(TableOperations::AddRows(rows.to_vec()));
                
                let selection_model = unsafe { table_view.as_mut().unwrap().selection_model() };
                unsafe { selection_model.as_mut().unwrap().clear(); }
                for row in rows.iter().rev() {
                    let model_index_filtered = unsafe { filter_model.as_ref().unwrap().map_from_source(&model.as_mut().unwrap().index((*row, 0))) };
                    if model_index_filtered.is_valid() {
                        unsafe { selection_model.as_mut().unwrap().select((
                            &model_index_filtered,
                            Flags::from_enum(SelectionFlag::Select) | Flags::from_enum(SelectionFlag::Rows)
                        )); }
                    }
                }
                
                // Trick to tell the model to update everything.
                *undo_lock.borrow_mut() = true;
                unsafe { model.as_mut().unwrap().item((0, 0)).as_mut().unwrap().set_data((&Variant::new0(()), 16)); }
                *undo_lock.borrow_mut() = false;
            }

            TableOperations::SmartDelete((edits, rows)) => {

                // Re-insert all the removed rows and restore all the edits.
                *undo_lock.borrow_mut() = true;
                unsafe { rows.iter().rev().for_each(|x| model.as_mut().unwrap().insert_row((x.0, &x.1))); }
                let edits_before = unsafe { edits.iter().map(|x| (((x.0).0, (x.0).1), (&*model.as_mut().unwrap().item(((x.0).0, (x.0).1))).clone())).collect::<Vec<((i32, i32), *mut StandardItem)>>() };
                unsafe { edits.iter().for_each(|x| model.as_mut().unwrap().set_item(((x.0).0, (x.0).1, x.1.clone()))); }
                *undo_lock.borrow_mut() = false;

                // Prepare the redo operation.
                history_opposite.borrow_mut().push(TableOperations::RevertSmartDelete((edits_before, rows.iter().map(|x| x.0).collect())));

                // Select all the edited items/rows.
                let selection_model = unsafe { table_view.as_mut().unwrap().selection_model() };
                unsafe { selection_model.as_mut().unwrap().clear(); }
                for row in rows.iter().rev() {
                    let model_index_filtered = unsafe { filter_model.as_ref().unwrap().map_from_source(&model.as_mut().unwrap().index((row.0, 0))) };
                    if model_index_filtered.is_valid() {
                        unsafe { selection_model.as_mut().unwrap().select((
                            &model_index_filtered,
                            Flags::from_enum(SelectionFlag::Select) | Flags::from_enum(SelectionFlag::Rows)
                        )); }
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
            TableOperations::RevertSmartDelete((edits, rows)) => {
                
                // Remake all the "edits" and remove the restored rows.
                let edits_before = unsafe { edits.iter().map(|x| (((x.0).0, (x.0).1), (&*model.as_mut().unwrap().item(((x.0).0, (x.0).1))).clone())).collect::<Vec<((i32, i32), *mut StandardItem)>>() };
                unsafe { edits.iter().for_each(|x| model.as_mut().unwrap().set_item(((x.0).0, (x.0).1, x.1.clone()))); }
                let mut removed_rows = vec![];
                unsafe { rows.iter().for_each(|x| removed_rows.push((*x, model.as_mut().unwrap().take_row(*x)))); }

                // Prepare the redo operation.
                history_opposite.borrow_mut().push(TableOperations::SmartDelete((edits_before, removed_rows)));

                // Select all the edited items, if any.
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

                // Try to save the PackedFile to the main PackFile.
                let _y = Self::save_to_packed_file(
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
            TableOperations::ImportTSVDB(table_data) => {

                // Prepare the redo operation.
                history_opposite.borrow_mut().push(TableOperations::ImportTSVDB(data.borrow().clone()));

                Self::load_data_to_table_view(&dependency_data, &table_data, table_view, model);
                build_columns(&data.borrow().table_definition, table_view, model);

                // If we want to let the columns resize themselfs...
                if *SETTINGS.lock().unwrap().settings_bool.get("adjust_columns_to_content").unwrap() {
                    unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
                }

                // Try to save the PackedFile to the main PackFile.
                let _y = Self::save_to_packed_file(
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

/// Implementation of PackedFileDBDecoder.
impl PackedFileDBDecoder {

    /// This functin returns a dummy struct. Use it for initialization.
    pub fn new() -> Self {

        // Create some dummy slots and return it.
        Self {
            slot_hex_view_scroll_sync: SlotCInt::new(|_| {}),
            slot_hex_view_raw_selection_sync: SlotNoArgs::new(|| {}),
            slot_hex_view_decoded_selection_sync: SlotNoArgs::new(|| {}),
            slot_hex_view_raw_selection_decoding: SlotNoArgs::new(|| {}),
            slot_hex_view_decoded_selection_decoding: SlotNoArgs::new(|| {}),
            slot_use_this_bool: SlotNoArgs::new(|| {}),
            slot_use_this_float: SlotNoArgs::new(|| {}),
            slot_use_this_integer: SlotNoArgs::new(|| {}),
            slot_use_this_long_integer: SlotNoArgs::new(|| {}),
            slot_use_this_string_u8: SlotNoArgs::new(|| {}),
            slot_use_this_string_u16: SlotNoArgs::new(|| {}),
            slot_use_this_optional_string_u8: SlotNoArgs::new(|| {}),
            slot_use_this_optional_string_u16: SlotNoArgs::new(|| {}),
            slot_table_change_field_type: SlotModelIndexRefModelIndexRefVectorVectorCIntRef::new(|_,_,_| {}),
            slot_table_view_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(|_,_| {}),
            slot_table_view_context_menu: SlotQtCorePointRef::new(|_| {}),
            slot_table_view_context_menu_move_up: SlotBool::new(|_| {}),
            slot_table_view_context_menu_move_down: SlotBool::new(|_| {}),
            slot_table_view_context_menu_delete: SlotBool::new(|_| {}),
            slot_remove_all_fields: SlotNoArgs::new(|| {}),
            slot_save_definition: SlotNoArgs::new(|| {}),
            slot_table_view_old_versions_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(|_,_| {}),
            slot_table_view_old_versions_context_menu: SlotQtCorePointRef::new(|_| {}),
            slot_table_view_old_versions_context_menu_load: SlotBool::new(|_| {}),
            slot_table_view_old_versions_context_menu_delete: SlotBool::new(|_| {}),
        }
    }

    /// This function creates the "Decoder View" with all the stuff needed to decode a table, and it
    /// returns it if it succeed. It can fail if the provided PackedFile is not a DB Table.
    pub fn create_decoder_view(
        sender_qt: Sender<Commands>,
        sender_qt_data: &Sender<Data>,
        receiver_qt: &Rc<RefCell<Receiver<Data>>>,
        app_ui: &AppUI,
        packed_file_path: &[String],
    ) -> Result<(Self, Font)> {

        //---------------------------------------------------------------------------------------//
        // Create the UI of the Decoder View...
        //---------------------------------------------------------------------------------------//

        // Create the widget that'll act as a container for the view.
        let widget = Widget::new().into_raw();
        let widget_layout = GridLayout::new().into_raw();
        unsafe { widget.as_mut().unwrap().set_layout(widget_layout as *mut Layout); }
        unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(0, widget); }

        // Create the hex view on the left side.
        let hex_view_group = GroupBox::new(&QString::from_std_str("PackedFile's Data")).into_raw();
        let hex_view_index = TextEdit::new(()).into_raw();
        let hex_view_raw = TextEdit::new(()).into_raw();
        let hex_view_decoded = TextEdit::new(()).into_raw();
        let hex_view_layout = GridLayout::new().into_raw();
        unsafe { hex_view_group.as_mut().unwrap().set_layout(hex_view_layout as *mut Layout); }
        unsafe { hex_view_layout.as_mut().unwrap().set_spacing(1); }
        unsafe { hex_view_layout.as_mut().unwrap().add_widget((hex_view_index as *mut Widget, 0, 0, 1, 1)); }
        unsafe { hex_view_layout.as_mut().unwrap().add_widget((hex_view_raw as *mut Widget, 0, 1, 1, 1)); }
        unsafe { hex_view_layout.as_mut().unwrap().add_widget((hex_view_decoded as *mut Widget, 0, 2, 1, 1)); }

        // Set them as "ReadOnly".
        unsafe { hex_view_index.as_mut().unwrap().set_read_only(true); }
        unsafe { hex_view_raw.as_mut().unwrap().set_read_only(true); }
        unsafe { hex_view_decoded.as_mut().unwrap().set_read_only(true); }

        // Set his font as "Monospace", so all the lines have the same lenght.
        let mut monospace_font = Font::new(&QString::from_std_str("Monospace"));
        monospace_font.set_style_hint(StyleHint::Monospace);
        unsafe { hex_view_index.as_mut().unwrap().set_font(&monospace_font); }
        unsafe { hex_view_raw.as_mut().unwrap().set_font(&monospace_font); }
        unsafe { hex_view_decoded.as_mut().unwrap().set_font(&monospace_font); }

        // Create the TableView at the top.
        let table_view = TableView::new().into_raw();
        let table_model = StandardItemModel::new(()).into_raw();
        unsafe { table_view.as_mut().unwrap().set_model(table_model as *mut AbstractItemModel); }
        unsafe { table_view.as_mut().unwrap().set_context_menu_policy(ContextMenuPolicy::Custom); }
        unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }

        // Create the Contextual Menu for the TableView.
        let mut table_view_context_menu = Menu::new(());

        // Create the Contextual Menu Actions.
        let table_view_context_menu_move_up = table_view_context_menu.add_action(&QString::from_std_str("Move &Up"));
        let table_view_context_menu_move_down = table_view_context_menu.add_action(&QString::from_std_str("&Move Down"));
        let table_view_context_menu_delete = table_view_context_menu.add_action(&QString::from_std_str("&Delete"));

        // Set the shortcuts for these actions.
        unsafe { table_view_context_menu_move_up.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().db_decoder_fields.get("move_up").unwrap()))); }
        unsafe { table_view_context_menu_move_down.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().db_decoder_fields.get("move_down").unwrap()))); }
        unsafe { table_view_context_menu_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().db_decoder_fields.get("delete").unwrap()))); }

        // Set the shortcuts to only trigger in the TableView.
        unsafe { table_view_context_menu_move_up.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { table_view_context_menu_move_down.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { table_view_context_menu_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }

        // Add them to the TableView.
        unsafe { table_view.as_mut().unwrap().add_action(table_view_context_menu_move_up); }
        unsafe { table_view.as_mut().unwrap().add_action(table_view_context_menu_move_down); }
        unsafe { table_view.as_mut().unwrap().add_action(table_view_context_menu_delete); }

        // Disable them by default.
        unsafe { table_view_context_menu_move_up.as_mut().unwrap().set_enabled(false); }
        unsafe { table_view_context_menu_move_down.as_mut().unwrap().set_enabled(false); }
        unsafe { table_view_context_menu_delete.as_mut().unwrap().set_enabled(false); }

        // Create the fields splitter.
        let fields_splitter = Splitter::new(Orientation::Vertical).into_raw();
        unsafe { fields_splitter.as_mut().unwrap().set_collapsible(0, false); }
        unsafe { fields_splitter.as_mut().unwrap().set_collapsible(1, false); }

        // Create the frames for the info.
        let decoded_fields_frame = GroupBox::new(&QString::from_std_str("Current Field Decoded")).into_raw();
        let selected_fields_frame = GroupBox::new(&QString::from_std_str("Selected Field Decoded")).into_raw();
        let info_frame = GroupBox::new(&QString::from_std_str("Table Info")).into_raw();

        // Add the stuff to the splitter.
        unsafe { fields_splitter.as_mut().unwrap().add_widget(decoded_fields_frame as *mut Widget); }
        unsafe { fields_splitter.as_mut().unwrap().add_widget(selected_fields_frame as *mut Widget); }

        // Set their layouts.
        let decoded_fields_layout = GridLayout::new().into_raw();
        let selected_fields_layout = GridLayout::new().into_raw();
        let info_layout = GridLayout::new().into_raw();
        unsafe { decoded_fields_layout.as_mut().unwrap().set_column_stretch(1, 10); }
        unsafe { selected_fields_layout.as_mut().unwrap().set_column_stretch(1, 10); }
        unsafe { decoded_fields_frame.as_mut().unwrap().set_layout(decoded_fields_layout as *mut Layout); }
        unsafe { selected_fields_frame.as_mut().unwrap().set_layout(selected_fields_layout as *mut Layout); }
        unsafe { info_frame.as_mut().unwrap().set_layout(info_layout as *mut Layout); }

        // Create the stuff for the decoded fields.
        let bool_label = Label::new(&QString::from_std_str("Decoded as \"Bool\":")).into_raw();
        let float_label = Label::new(&QString::from_std_str("Decoded as \"Float\":")).into_raw();
        let integer_label = Label::new(&QString::from_std_str("Decoded as \"Integer\":")).into_raw();
        let long_integer_label = Label::new(&QString::from_std_str("Decoded as \"Long Integer\":")).into_raw();
        let string_u8_label = Label::new(&QString::from_std_str("Decoded as \"String U8\":")).into_raw();
        let string_u16_label = Label::new(&QString::from_std_str("Decoded as \"String U16\":")).into_raw();
        let optional_string_u8_label = Label::new(&QString::from_std_str("Decoded as \"Optional String U8\":")).into_raw();
        let optional_string_u16_label = Label::new(&QString::from_std_str("Decoded as \"Optional String U16\":")).into_raw();

        let bool_line_edit = LineEdit::new(()).into_raw();
        let float_line_edit = LineEdit::new(()).into_raw();
        let integer_line_edit = LineEdit::new(()).into_raw();
        let long_integer_line_edit = LineEdit::new(()).into_raw();
        let string_u8_line_edit = LineEdit::new(()).into_raw();
        let string_u16_line_edit = LineEdit::new(()).into_raw();
        let optional_string_u8_line_edit = LineEdit::new(()).into_raw();
        let optional_string_u16_line_edit = LineEdit::new(()).into_raw();

        let bool_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let float_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let integer_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let long_integer_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let string_u8_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let string_u16_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let optional_string_u8_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let optional_string_u16_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();

        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((bool_label as *mut Widget, 0, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((float_label as *mut Widget, 1, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((integer_label as *mut Widget, 2, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((long_integer_label as *mut Widget, 3, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u8_label as *mut Widget, 4, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u16_label as *mut Widget, 5, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u8_label as *mut Widget, 6, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u16_label as *mut Widget, 7, 0, 1, 1)); }

        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((bool_line_edit as *mut Widget, 0, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((float_line_edit as *mut Widget, 1, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((integer_line_edit as *mut Widget, 2, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((long_integer_line_edit as *mut Widget, 3, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u8_line_edit as *mut Widget, 4, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u16_line_edit as *mut Widget, 5, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u8_line_edit as *mut Widget, 6, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u16_line_edit as *mut Widget, 7, 1, 1, 1)); }

        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((bool_button as *mut Widget, 0, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((float_button as *mut Widget, 1, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((integer_button as *mut Widget, 2, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((long_integer_button as *mut Widget, 3, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u8_button as *mut Widget, 4, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u16_button as *mut Widget, 5, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u8_button as *mut Widget, 6, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u16_button as *mut Widget, 7, 2, 1, 1)); }

        // Create the stuff for the "Selection" live decoding.
        let selection_bool_label = Label::new(&QString::from_std_str("Decoded as \"Bool\":")).into_raw();
        let selection_float_label = Label::new(&QString::from_std_str("Decoded as \"Float\":")).into_raw();
        let selection_integer_label = Label::new(&QString::from_std_str("Decoded as \"Integer\":")).into_raw();
        let selection_long_integer_label = Label::new(&QString::from_std_str("Decoded as \"Long Integer\":")).into_raw();
        let selection_string_u8_label = Label::new(&QString::from_std_str("Decoded as \"String U8\":")).into_raw();
        let selection_string_u16_label = Label::new(&QString::from_std_str("Decoded as \"String U16\":")).into_raw();
        let selection_optional_string_u8_label = Label::new(&QString::from_std_str("Decoded as \"Optional String U8\":")).into_raw();
        let selection_optional_string_u16_label = Label::new(&QString::from_std_str("Decoded as \"Optional String U16\":")).into_raw();

        let selection_bool_line_edit = LineEdit::new(()).into_raw();
        let selection_float_line_edit = LineEdit::new(()).into_raw();
        let selection_integer_line_edit = LineEdit::new(()).into_raw();
        let selection_long_integer_line_edit = LineEdit::new(()).into_raw();
        let selection_string_u8_line_edit = LineEdit::new(()).into_raw();
        let selection_string_u16_line_edit = LineEdit::new(()).into_raw();
        let selection_optional_string_u8_line_edit = LineEdit::new(()).into_raw();
        let selection_optional_string_u16_line_edit = LineEdit::new(()).into_raw();

        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_bool_label as *mut Widget, 0, 0, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_float_label as *mut Widget, 1, 0, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_integer_label as *mut Widget, 2, 0, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_long_integer_label as *mut Widget, 3, 0, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_string_u8_label as *mut Widget, 4, 0, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_string_u16_label as *mut Widget, 5, 0, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_optional_string_u8_label as *mut Widget, 6, 0, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_optional_string_u16_label as *mut Widget, 7, 0, 1, 1)); }

        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_bool_line_edit as *mut Widget, 0, 1, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_float_line_edit as *mut Widget, 1, 1, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_integer_line_edit as *mut Widget, 2, 1, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_long_integer_line_edit as *mut Widget, 3, 1, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_string_u8_line_edit as *mut Widget, 4, 1, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_string_u16_line_edit as *mut Widget, 5, 1, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_optional_string_u8_line_edit as *mut Widget, 6, 1, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_optional_string_u16_line_edit as *mut Widget, 7, 1, 1, 1)); }

        // Create stuff for the info frame.
        let table_info_type_label = Label::new(&QString::from_std_str("Table type:")).into_raw();
        let table_info_version_label = Label::new(&QString::from_std_str("Table version:")).into_raw();
        let table_info_entry_count_label = Label::new(&QString::from_std_str("Table entry count:")).into_raw();

        let table_info_type_decoded_label = Label::new(()).into_raw();
        let table_info_version_decoded_label = Label::new(()).into_raw();
        let table_info_entry_count_decoded_label = Label::new(()).into_raw();

        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_type_label as *mut Widget, 0, 0, 1, 1)); }
        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_version_label as *mut Widget, 1, 0, 1, 1)); }
        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_entry_count_label as *mut Widget, 2, 0, 1, 1)); }

        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_type_decoded_label as *mut Widget, 0, 1, 1, 1)); }
        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_version_decoded_label as *mut Widget, 1, 1, 1, 1)); }
        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_entry_count_decoded_label as *mut Widget, 2, 1, 1, 1)); }

        // Create the TableView at the top.
        let table_view_old_versions = TableView::new().into_raw();
        let table_model_old_versions = StandardItemModel::new(()).into_raw();
        unsafe { table_view_old_versions.as_mut().unwrap().set_model(table_model_old_versions as *mut AbstractItemModel); }

        // Set it as not editable.
        unsafe { table_view_old_versions.as_mut().unwrap().set_edit_triggers(Flags::from_enum(EditTrigger::NoEditTriggers)); };
        unsafe { table_view_old_versions.as_mut().unwrap().set_selection_mode(SelectionMode::Single); };

        // Sort the versions.
        unsafe { table_view_old_versions.as_mut().unwrap().set_sorting_enabled(true); }
        unsafe { table_view_old_versions.as_mut().unwrap().sort_by_column((0, SortOrder::Ascending)); }

        // Hide the vertical header.
        unsafe { table_view_old_versions.as_mut().unwrap().vertical_header().as_mut().unwrap().set_visible(false); }

        // Prepare it for the Context Menu.
        unsafe { table_view_old_versions.as_mut().unwrap().set_context_menu_policy(ContextMenuPolicy::Custom); }

        // Create the Contextual Menu for the TableView.
        let mut table_view_old_versions_context_menu = Menu::new(());

        // Create the Contextual Menu Actions.
        let table_view_old_versions_context_menu_load = table_view_old_versions_context_menu.add_action(&QString::from_std_str("&Load"));
        let table_view_old_versions_context_menu_delete = table_view_old_versions_context_menu.add_action(&QString::from_std_str("&Delete"));

        // Set the shortcuts for these actions.
        unsafe { table_view_old_versions_context_menu_load.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().db_decoder_definitions.get("load").unwrap()))); }
        unsafe { table_view_old_versions_context_menu_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(SHORTCUTS.lock().unwrap().db_decoder_definitions.get("delete").unwrap()))); }

        // Set the shortcuts to only trigger in the TableView.
        unsafe { table_view_old_versions_context_menu_load.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { table_view_old_versions_context_menu_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }

        // Add them to the TableView.
        unsafe { table_view_old_versions.as_mut().unwrap().add_action(table_view_old_versions_context_menu_load); }
        unsafe { table_view_old_versions.as_mut().unwrap().add_action(table_view_old_versions_context_menu_delete); }

        // Disable them by default.
        unsafe { table_view_old_versions_context_menu_load.as_mut().unwrap().set_enabled(false); }
        unsafe { table_view_old_versions_context_menu_delete.as_mut().unwrap().set_enabled(false); }

        // Create the bottom ButtonBox.
        let button_box = Frame::new().into_raw();
        let button_box_layout = GridLayout::new().into_raw();
        unsafe { button_box.as_mut().unwrap().set_layout(button_box_layout as *mut Layout); }

        // Create the bottom Buttons.
        let clear_definition_button = PushButton::new(&QString::from_std_str("Remove all fields")).into_raw();
        let save_button = PushButton::new(&QString::from_std_str("Finish it!")).into_raw();

        // Add them to the Dialog.
        unsafe { button_box_layout.as_mut().unwrap().add_widget((clear_definition_button as *mut Widget, 0, 0, 1, 1)); }
        unsafe { button_box_layout.as_mut().unwrap().add_widget((save_button as *mut Widget, 0, 1, 1, 1)); }

        // Add everything to the main grid.
        unsafe { widget_layout.as_mut().unwrap().add_widget((hex_view_group as *mut Widget, 0, 0, 5, 1)); }
        unsafe { widget_layout.as_mut().unwrap().add_widget((table_view as *mut Widget, 0, 1, 1, 2)); }
        unsafe { widget_layout.as_mut().unwrap().add_widget((fields_splitter as *mut Widget, 1, 1, 4, 1)); }
        unsafe { widget_layout.as_mut().unwrap().add_widget((info_frame as *mut Widget, 1, 2, 1, 1)); }
        unsafe { widget_layout.as_mut().unwrap().add_widget((table_view_old_versions as *mut Widget, 2, 2, 2, 1)); }
        unsafe { widget_layout.as_mut().unwrap().add_widget((button_box as *mut Widget, 4, 2, 1, 1)); }
        unsafe { widget_layout.as_mut().unwrap().set_column_stretch(1, 10); }
        unsafe { widget_layout.as_mut().unwrap().set_row_stretch(0, 10); }
        unsafe { widget_layout.as_mut().unwrap().set_row_stretch(2, 5); }

        //---------------------------------------------------------------------------------------//
        // Prepare the data for the Decoder View...
        //---------------------------------------------------------------------------------------//

        // Get the PackedFile.
        sender_qt.send(Commands::GetPackedFile).unwrap();
        sender_qt_data.send(Data::VecString(packed_file_path.to_vec())).unwrap();
        let mut packed_file = match check_message_validity_recv2(&receiver_qt) {
            Data::PackedFile(data) => data,
            Data::Error(error) => return Err(error),
            _ => panic!(THREADS_MESSAGE_ERROR),
        };

        // If the PackedFile is in the db folder...
        if packed_file.path.len() == 3 {
            if packed_file.path[0] == "db" {

                // Put all together so we can pass it easely.
                let stuff = PackedFileDBDecoderStuff {
                    hex_view_index,
                    hex_view_raw,
                    hex_view_decoded,
                    table_view,
                    table_model,
                    selection_bool_line_edit,
                    selection_float_line_edit,
                    selection_integer_line_edit,
                    selection_long_integer_line_edit,
                    selection_string_u8_line_edit,
                    selection_string_u16_line_edit,
                    selection_optional_string_u8_line_edit,
                    selection_optional_string_u16_line_edit,
                    bool_line_edit,
                    float_line_edit,
                    integer_line_edit,
                    long_integer_line_edit,
                    string_u8_line_edit,
                    string_u16_line_edit,
                    optional_string_u8_line_edit,
                    optional_string_u16_line_edit,
                    bool_button,
                    float_button,
                    integer_button,
                    long_integer_button,
                    string_u8_button,
                    string_u16_button,
                    optional_string_u8_button,
                    optional_string_u16_button,
                    table_info_type_decoded_label,
                    table_info_version_decoded_label,
                    table_info_entry_count_decoded_label,
                    table_view_old_versions,
                    table_model_old_versions,
                    clear_definition_button,
                    save_button,
                    table_view_context_menu: table_view_context_menu.into_raw(),
                    table_view_context_menu_move_up,
                    table_view_context_menu_move_down,
                    table_view_context_menu_delete,
                    table_view_old_versions_context_menu: table_view_old_versions_context_menu.into_raw(),
                    table_view_old_versions_context_menu_load,
                    table_view_old_versions_context_menu_delete,
                };

                // Check if it can be read as a table.
                let packed_file_data = packed_file.get_data_and_keep_it()?;
                match DB::get_header_data(&packed_file_data) {

                    // If we succeed at decoding his header...
                    Ok((version, entry_count, initial_index)) => {

                        // Put all the "Non UI" data we need to keep together.
                        let stuff_non_ui = PackedFileDBDecoderStuffNonUI {
                            packed_file_path: packed_file.path.to_vec(),
                            packed_file_data,
                            initial_index,
                            version,
                            entry_count,
                        };

                        // Get the index we are going to "manipulate".
                        let index = Rc::new(RefCell::new(stuff_non_ui.initial_index));

                        // Check if we have an schema.
                        let mut schema = SCHEMA.lock().unwrap().clone();
                        match schema {

                            // If we have an schema...
                            Some(schema) => {

                                // Get the table definition for this table (or create a new one).
                                let table_definition = match DB::get_schema(&stuff_non_ui.packed_file_path[1], stuff_non_ui.version, &schema) {
                                    Some(table_definition) => Rc::new(RefCell::new(table_definition)),
                                    None => Rc::new(RefCell::new(TableDefinition::new(stuff_non_ui.version)))
                                };

                                //---------------------------------------------------------------------------------------//
                                // Load the data to the Decoder View...
                                //---------------------------------------------------------------------------------------//

                                // Load the static data into the Decoder View.
                                Self::load_data_to_decoder_view(&stuff, &stuff_non_ui);

                                // Update the versions list.
                                Self::update_versions_list(&stuff, &schema, &stuff_non_ui.packed_file_path[1]);

                                // Update the Decoder View's Dynamic Data (LineEdits, Table,...) and recalculate
                                // the current "index_data" (position in the vector we are decoding).
                                Self::update_decoder_view(
                                    &stuff, &stuff_non_ui,
                                    (true, &table_definition.borrow().fields),
                                    &mut index.borrow_mut()
                                );

                                // Put the schema into a Rc<RefCell<Schema>>, so we can modify it.
                                let schema = Rc::new(RefCell::new(schema));

                                //---------------------------------------------------------------------------------------//
                                // Create the slots for the decoder view...
                                //---------------------------------------------------------------------------------------//

                                // Create all the slots we need to keep alive later on.
                                let slots = Self {

                                    // Slot to sync all the TextEdit in the "Hex Data" area.
                                    slot_hex_view_scroll_sync: SlotCInt::new(clone!(
                                        stuff => move |value| {
                                            unsafe { stuff.hex_view_index.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().set_value(value); }
                                            unsafe { stuff.hex_view_raw.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().set_value(value); }
                                            unsafe { stuff.hex_view_decoded.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().set_value(value); }
                                        }
                                    )),

                                    // Slot to sync the selection of both HexViews (Raw => Decoded).
                                    slot_hex_view_raw_selection_sync: SlotNoArgs::new(clone!(
                                        stuff => move || {

                                            // Get the cursor of the TextEdit.
                                            let cursor;
                                            let cursor_dest;
                                            unsafe { cursor = stuff.hex_view_raw.as_mut().unwrap().text_cursor().into_raw(); }
                                            unsafe { cursor_dest = stuff.hex_view_decoded.as_mut().unwrap().text_cursor().into_raw(); }

                                            // Get the limits of the selection.
                                            let mut selection_start;
                                            let mut selection_end;
                                            unsafe { selection_start = cursor.as_mut().unwrap().selection_start(); }
                                            unsafe { selection_end = cursor.as_mut().unwrap().selection_end(); }

                                            // Translate those limits to fit the other HexView.
                                            selection_start = ((selection_start + 1) / 3) + (selection_start / 48);
                                            selection_end = ((selection_end + 2) / 3) + (selection_end / 48);

                                            // If we got something selected, always select something in the other HexView.
                                            unsafe { if selection_start == selection_end && cursor.as_mut().unwrap().selection_start() != cursor.as_mut().unwrap().selection_end() { selection_end += 1; } }

                                            // Select the corresponding lines in the other HexView.
                                            unsafe { cursor_dest.as_mut().unwrap().move_position(MoveOperation::Start); }
                                            unsafe { cursor_dest.as_mut().unwrap().move_position((MoveOperation::NextCharacter, MoveMode::Move, selection_start as i32)); }
                                            unsafe { cursor_dest.as_mut().unwrap().move_position((MoveOperation::NextCharacter, MoveMode::Keep, (selection_end - selection_start) as i32)); }

                                            // Block the signals during this, so we don't trigger an infinite loop.
                                            let mut blocker;
                                            unsafe { blocker = SignalBlocker::new(stuff.hex_view_decoded.as_mut().unwrap().static_cast_mut() as &mut Object); }
                                            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor_dest.as_ref().unwrap()); }
                                            blocker.unblock();
                                        }
                                    )),

                                    // Slot to sync the selection of both HexViews (Decoded => Raw).
                                    slot_hex_view_decoded_selection_sync: SlotNoArgs::new(clone!(
                                        stuff => move || {

                                            // Get the cursor of the TextEdit.
                                            let cursor;
                                            let cursor_dest;
                                            unsafe { cursor = stuff.hex_view_decoded.as_mut().unwrap().text_cursor().into_raw(); }
                                            unsafe { cursor_dest = stuff.hex_view_raw.as_mut().unwrap().text_cursor().into_raw(); }

                                            // Get the limits of the selection.
                                            let mut selection_start;
                                            let mut selection_end;
                                            unsafe { selection_start = cursor.as_mut().unwrap().selection_start(); }
                                            unsafe { selection_end = cursor.as_mut().unwrap().selection_end(); }

                                            // Translate those limits to fit the other HexView.
                                            selection_start = (selection_start - (selection_start / 17)) * 3;
                                            selection_end = (selection_end - (selection_end / 17)) * 3;

                                            // Select the corresponding lines in the other HexView.
                                            unsafe { cursor_dest.as_mut().unwrap().move_position(MoveOperation::Start); }
                                            unsafe { cursor_dest.as_mut().unwrap().move_position((MoveOperation::NextCharacter, MoveMode::Move, selection_start as i32)); }
                                            unsafe { cursor_dest.as_mut().unwrap().move_position((MoveOperation::NextCharacter, MoveMode::Keep, (selection_end - selection_start) as i32)); }

                                            // Block the signals during this, so we don't trigger an infinite loop.
                                            let mut blocker;
                                            unsafe { blocker = SignalBlocker::new(stuff.hex_view_raw.as_mut().unwrap().static_cast_mut() as &mut Object); }
                                            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor_dest.as_ref().unwrap()); }
                                            blocker.unblock();
                                        }
                                    )),

                                    // Slot to get the selected text and decode it on-the-fly from the HexView (Raw).
                                    slot_hex_view_raw_selection_decoding: SlotNoArgs::new(clone!(
                                        stuff_non_ui,
                                        stuff => move || {

                                            // Get the cursor of the TextEdit.
                                            let cursor;
                                            unsafe { cursor = stuff.hex_view_raw.as_mut().unwrap().text_cursor().into_raw(); }

                                            // Get the limits of the selection.
                                            let selection_start;
                                            unsafe { selection_start = ((cursor.as_mut().unwrap().selection_start()) / 3) as usize; }

                                            // Update the LineEdits.
                                            Self::update_selection_decoded_fields(&stuff, &stuff_non_ui, selection_start);
                                        }
                                    )),

                                    // Slot to get the selected text and decode it on-the-fly from the HexView (Decoded).
                                    slot_hex_view_decoded_selection_decoding: SlotNoArgs::new(clone!(
                                        stuff_non_ui,
                                        stuff => move || {

                                            // Get the cursor of the TextEdit.
                                            let cursor;
                                            unsafe { cursor = stuff.hex_view_decoded.as_mut().unwrap().text_cursor().into_raw(); }

                                            // Get the limits of the selection.
                                            let selection_start;
                                            unsafe { selection_start = (cursor.as_mut().unwrap().selection_start() - (cursor.as_mut().unwrap().selection_start() / 17)) as usize; }

                                            // Update the LineEdits.
                                            Self::update_selection_decoded_fields(&stuff, &stuff_non_ui, selection_start);
                                        }
                                    )),

                                    // Slots for the "Use this" buttons.
                                    slot_use_this_bool: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::Boolean, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_float: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::Float, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_integer: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::Integer, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_long_integer: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::LongInteger, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_string_u8: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::StringU8, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_string_u16: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::StringU16, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_optional_string_u8: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::OptionalStringU8, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_optional_string_u16: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::OptionalStringU16, &mut index.borrow_mut());
                                        }
                                    )),

                                    // Slot for when we change the Type of the selected field in the table.
                                    slot_table_change_field_type: SlotModelIndexRefModelIndexRefVectorVectorCIntRef::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move |initial_model_index,final_model_index,_| {

                                            // If both changed cells are from the Type column...
                                            if initial_model_index.column() == 1 && final_model_index.column() == 1 {

                                                // Update the "First row decoded" column, and get the new "index" to continue decoding.
                                                let invalid_rows = Self::update_first_row_decoded(&stuff, &stuff_non_ui, &mut index.borrow_mut());

                                                // Fix the broken rows.
                                                for row in &invalid_rows {

                                                    // Get the item from the type column.
                                                    let item;
                                                    unsafe { item = stuff.table_model.as_mut().unwrap().item((*row, 1)); }

                                                    // Change it to bool.
                                                    unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str("Bool")); }
                                                }
                                            }
                                        }
                                    )),

                                    // Slot to enable/disable contextual actions depending on the selected item.
                                    slot_table_view_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(clone!(
                                        stuff => move |selection,_| {

                                            // If there is something selected...
                                            if !selection.indexes().is_empty() {
                                                unsafe { stuff.table_view_context_menu_move_up.as_mut().unwrap().set_enabled(true); }
                                                unsafe { stuff.table_view_context_menu_move_down.as_mut().unwrap().set_enabled(true); }
                                                unsafe { stuff.table_view_context_menu_delete.as_mut().unwrap().set_enabled(true); }
                                            }

                                            // Otherwise, disable everything.
                                            else {
                                                unsafe { stuff.table_view_context_menu_move_up.as_mut().unwrap().set_enabled(false); }
                                                unsafe { stuff.table_view_context_menu_move_down.as_mut().unwrap().set_enabled(false); }
                                                unsafe { stuff.table_view_context_menu_delete.as_mut().unwrap().set_enabled(false); }
                                            }
                                        }
                                    )),

                                    // Slot to show the Contextual Menu for the TableView.
                                    slot_table_view_context_menu: SlotQtCorePointRef::new(clone!(
                                        stuff => move |_| {
                                            unsafe { stuff.table_view_context_menu.as_mut().unwrap().exec2(&Cursor::pos()); }
                                        }
                                    )),

                                    // Slots for the Contextual Menu Actions of the TableView.
                                    slot_table_view_context_menu_move_up: SlotBool::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move |_| {

                                            // Get the selection of the TableView.
                                            let selection;
                                            unsafe { selection = stuff.table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                                            let indexes = selection.indexes();

                                            //  Get the list of rows to move up.
                                            let mut rows = vec![];
                                            for index in 0..indexes.count(()) {
                                                rows.push(indexes.at(index).row());
                                            }

                                            // Dedup the list.
                                            rows.sort();
                                            rows.dedup();

                                            // For each row we have to move...
                                            for row in rows {

                                                // If we are in the row 0, skip it.
                                                if row == 0 { continue; }

                                                // Otherwise...
                                                else {

                                                    // Take the row from the table.
                                                    let row_data;
                                                    unsafe { row_data = stuff.table_model.as_mut().unwrap().take_row(row - 1); }

                                                    // Insert it one position above.
                                                    unsafe { stuff.table_model.as_mut().unwrap().insert_row((row, &row_data)); }
                                                }
                                            }

                                            // Update the "First row decoded" column.
                                            Self::update_first_row_decoded(&stuff, &stuff_non_ui, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_table_view_context_menu_move_down: SlotBool::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move |_| {

                                            // Get the selection of the TableView.
                                            let selection;
                                            unsafe { selection = stuff.table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                                            let indexes = selection.indexes();

                                            //  Get the list of rows to move down.
                                            let mut rows = vec![];
                                            for index in 0..indexes.count(()) {
                                                rows.push(indexes.at(index).row());
                                            }

                                            // Dedup the list and reverse it.
                                            rows.sort();
                                            rows.dedup();
                                            rows.reverse();

                                            // For each row we have to move...
                                            for row in rows {

                                                // Get the amount of rows in the Model.
                                                let row_count;
                                                unsafe { row_count = stuff.table_model.as_mut().unwrap().row_count(()); }

                                                // If we are in the last row, skip it.
                                                if row == (row_count - 1) { continue; }

                                                // Otherwise...
                                                else {

                                                    // Take the row from the table.
                                                    let row_data;
                                                    unsafe { row_data = stuff.table_model.as_mut().unwrap().take_row(row + 1); }

                                                    // Insert it one position above.
                                                    unsafe { stuff.table_model.as_mut().unwrap().insert_row((row, &row_data)); }
                                                }
                                            }

                                            // Update the "First row decoded" column.
                                            Self::update_first_row_decoded(&stuff, &stuff_non_ui, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_table_view_context_menu_delete: SlotBool::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move |_| {

                                            // Get the selection of the TableView.
                                            let selection;
                                            unsafe { selection = stuff.table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                                            let indexes = selection.indexes();

                                            //  Get the list of rows to remove.
                                            let mut rows = vec![];
                                            for index in 0..indexes.count(()) {
                                                rows.push(indexes.at(index).row());
                                            }

                                            // Dedup the list and reverse it.
                                            rows.sort();
                                            rows.dedup();
                                            rows.reverse();

                                            // For each row we have to remove...
                                            for row in rows {

                                                // Remove it.
                                                unsafe { stuff.table_model.as_mut().unwrap().remove_row(row); }
                                            }

                                            // Update the "First row decoded" column.
                                            Self::update_first_row_decoded(&stuff, &stuff_non_ui, &mut index.borrow_mut());
                                        }
                                    )),

                                    // Slot for the "Kill them all!" button.
                                    slot_remove_all_fields: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {

                                            // Remove everything from the model.
                                            unsafe { stuff.table_model.as_mut().unwrap().clear(); }

                                            // Reset the index.
                                            *index.borrow_mut() = stuff_non_ui.initial_index;

                                            // Update the decoder view.
                                            Self::update_decoder_view(&stuff, &stuff_non_ui, (false, &[]), &mut index.borrow_mut());
                                        }
                                    )),

                                    // Slot for the "Finish it!" button.
                                    slot_save_definition: SlotNoArgs::new(clone!(
                                        sender_qt,
                                        sender_qt_data,
                                        receiver_qt,
                                        table_definition,
                                        schema,
                                        app_ui,
                                        stuff,
                                        stuff_non_ui => move || {

                                            // We get the index of our table's definitions. In case we find it, we just return it. If it's not
                                            // the case, then we create a new table's definitions and return his index. To know if we didn't found
                                            // an index, we just return -1 as index.
                                            let mut table_definitions_index = match schema.borrow().get_table_definitions(&stuff_non_ui.packed_file_path[1]) {
                                                Some(table_definitions_index) => table_definitions_index as i32,
                                                None => -1i32,
                                            };

                                            // If we didn't found a table definition for our table...
                                            if table_definitions_index == -1 {

                                                // We create one.
                                                schema.borrow_mut().add_table_definitions(TableDefinitions::new(&stuff_non_ui.packed_file_path[1]));

                                                // And get his index.
                                                table_definitions_index = schema.borrow().get_table_definitions(&stuff_non_ui.packed_file_path[1]).unwrap() as i32;
                                            }

                                            // We replace his fields with the ones from the TableView.
                                            table_definition.borrow_mut().fields = Self::return_data_from_data_view(&stuff);

                                            // We add our `TableDefinition` to the main `Schema` and sort it, so the TableDefinition is in the right place.
                                            schema.borrow_mut().tables_definitions[table_definitions_index as usize].add_table_definition(table_definition.borrow().clone());
                                            schema.borrow_mut().tables_definitions.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                                            schema.borrow_mut().tables_definitions.iter_mut().for_each(|x| x.versions.sort_unstable_by(|a, b| b.version.cmp(&a.version)));

                                            // Send it back to the background thread for saving it.
                                            sender_qt.send(Commands::SaveSchema).unwrap();
                                            sender_qt_data.send(Data::Schema(schema.borrow().clone())).unwrap();

                                            // Report success while saving it, or an error.
                                            match check_message_validity_recv2(&receiver_qt) {
                                                Data::Success => show_dialog(app_ui.window, true, "Schema successfully saved."),
                                                Data::Error(error) => show_dialog(app_ui.window, false, error),
                                                _ => panic!(THREADS_MESSAGE_ERROR),
                                            }

                                            // After all that, we need to update the version list, as this may have created a new version.
                                            Self::update_versions_list(&stuff, &schema.borrow(), &stuff_non_ui.packed_file_path[1]);
                                        }
                                    )),

                                    // Actions to manage the Context Menu in the "Versions" TableView.
                                    slot_table_view_old_versions_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(clone!(
                                        stuff => move |selection,_| {

                                            // If there is only one version selected...
                                            if selection.indexes().count(()) == 1 {
                                                unsafe { stuff.table_view_old_versions_context_menu_load.as_mut().unwrap().set_enabled(true); }
                                                unsafe { stuff.table_view_old_versions_context_menu_delete.as_mut().unwrap().set_enabled(true); }
                                            }

                                            // Otherwise, disable everything.
                                            else {
                                                unsafe { stuff.table_view_old_versions_context_menu_load.as_mut().unwrap().set_enabled(false); }
                                                unsafe { stuff.table_view_old_versions_context_menu_delete.as_mut().unwrap().set_enabled(false); }
                                            }
                                        }
                                    )),
                                    slot_table_view_old_versions_context_menu: SlotQtCorePointRef::new(clone!(
                                        stuff => move |_| {
                                            unsafe { stuff.table_view_old_versions_context_menu.as_mut().unwrap().exec2(&Cursor::pos()); }
                                        }
                                    )),
                                    slot_table_view_old_versions_context_menu_load: SlotBool::new(clone!(
                                        schema,
                                        stuff,
                                        stuff_non_ui => move |_| {

                                            // Get the selection of the TableView.
                                            let selection;
                                            unsafe { selection = stuff.table_view_old_versions.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }

                                            // If we have something selected...
                                            if selection.indexes().count(()) == 1 {

                                                // Get the selected ModelIndex.
                                                let indexes = selection.indexes();
                                                let model_index = indexes.at(0);

                                                // Get the version selected.
                                                let version;
                                                unsafe { version = stuff.table_model_old_versions.as_mut().unwrap().item_from_index(&model_index).as_mut().unwrap().text().to_std_string(); }

                                                // Turn it into a number.
                                                let version = version.parse::<u32>().unwrap();

                                                // Get the new definition.
                                                let table_definition = DB::get_schema(&stuff_non_ui.packed_file_path[1], version, &*schema.borrow());

                                                // Remove everything from the model.
                                                unsafe { stuff.table_model.as_mut().unwrap().clear(); }

                                                // Reset the index.
                                                *index.borrow_mut() = stuff_non_ui.initial_index;

                                                // Update the decoder view.
                                                Self::update_decoder_view(&stuff, &stuff_non_ui, (true, &table_definition.unwrap().fields), &mut index.borrow_mut());
                                            }
                                        }
                                    )),
                                    slot_table_view_old_versions_context_menu_delete: SlotBool::new(clone!(
                                        schema,
                                        app_ui,
                                        stuff,
                                        stuff_non_ui => move |_| {

                                            // Get the selection of the TableView.
                                            let selection;
                                            unsafe { selection = stuff.table_view_old_versions.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }

                                            // If we have something selected...
                                            if selection.indexes().count(()) == 1 {

                                                // Get the selected ModelIndex.
                                                let indexes = selection.indexes();
                                                let model_index = indexes.at(0);

                                                // Get the version selected.
                                                let version;
                                                unsafe { version = stuff.table_model_old_versions.as_mut().unwrap().item_from_index(&model_index).as_mut().unwrap().text().to_std_string(); }

                                                // Turn it into a number.
                                                let version = version.parse::<u32>().unwrap();

                                                // Try to remove that version form the schema.
                                                if let Err(error) = DB::remove_table_version(&stuff_non_ui.packed_file_path[1], version, &mut schema.borrow_mut()) {
                                                    return show_dialog(app_ui.window, false, error.kind());
                                                }

                                                // If it worked, update the list.
                                                Self::update_versions_list(&stuff, &schema.borrow(), &stuff_non_ui.packed_file_path[1]);
                                            }
                                        }
                                    )),
                                };

                                // Sync the scroll bars of the three hex data views.
                                unsafe { stuff.hex_view_index.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().signals().value_changed().connect(&slots.slot_hex_view_scroll_sync); }
                                unsafe { stuff.hex_view_raw.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().signals().value_changed().connect(&slots.slot_hex_view_scroll_sync); }
                                unsafe { stuff.hex_view_decoded.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().signals().value_changed().connect(&slots.slot_hex_view_scroll_sync); }

                                // Decode on-the-fly whatever is selected in the HexView.
                                unsafe { stuff.hex_view_raw.as_mut().unwrap().signals().selection_changed().connect(&slots.slot_hex_view_raw_selection_decoding); }
                                unsafe { stuff.hex_view_decoded.as_mut().unwrap().signals().selection_changed().connect(&slots.slot_hex_view_decoded_selection_decoding); }

                                // Signal to sync the selection between both HexViews.
                                unsafe { stuff.hex_view_raw.as_mut().unwrap().signals().selection_changed().connect(&slots.slot_hex_view_raw_selection_sync); }
                                unsafe { stuff.hex_view_decoded.as_mut().unwrap().signals().selection_changed().connect(&slots.slot_hex_view_decoded_selection_sync); }

                                // Actions for the "Use this" buttons.
                                unsafe { stuff.bool_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_bool); }
                                unsafe { stuff.float_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_float); }
                                unsafe { stuff.integer_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_integer); }
                                unsafe { stuff.long_integer_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_long_integer); }
                                unsafe { stuff.string_u8_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_string_u8); }
                                unsafe { stuff.string_u16_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_string_u16); }
                                unsafe { stuff.optional_string_u8_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_optional_string_u8); }
                                unsafe { stuff.optional_string_u16_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_optional_string_u16); }

                                // Action for when we change the type of a field in the table.
                                unsafe { stuff.table_model.as_mut().unwrap().signals().data_changed().connect(&slots.slot_table_change_field_type); }

                                // Trigger the "Enable/Disable" slot every time we change the selection in the TableView.
                                unsafe { stuff.table_view.as_mut().unwrap().selection_model().as_ref().unwrap().signals().selection_changed().connect(&slots.slot_table_view_context_menu_enabler); }

                                // Action to show the Contextual Menu for the Field's TableView.
                                unsafe { (stuff.table_view as *mut Widget).as_ref().unwrap().signals().custom_context_menu_requested().connect(&slots.slot_table_view_context_menu); }

                                // Actions of the Contextual Menu for the Field's TableView.
                                unsafe { stuff.table_view_context_menu_move_up.as_mut().unwrap().signals().triggered().connect(&slots.slot_table_view_context_menu_move_up); }
                                unsafe { stuff.table_view_context_menu_move_down.as_mut().unwrap().signals().triggered().connect(&slots.slot_table_view_context_menu_move_down); }
                                unsafe { stuff.table_view_context_menu_delete.as_mut().unwrap().signals().triggered().connect(&slots.slot_table_view_context_menu_delete); }

                                // Action of the "Kill them all!" button.
                                unsafe { stuff.clear_definition_button.as_mut().unwrap().signals().released().connect(&slots.slot_remove_all_fields); }

                                // Action of the "Finish it!" button.
                                unsafe { stuff.save_button.as_mut().unwrap().signals().released().connect(&slots.slot_save_definition); }

                                // Actions for the Contextual Menu in the "Versions" table.
                                unsafe { stuff.table_view_old_versions.as_mut().unwrap().selection_model().as_ref().unwrap().signals().selection_changed().connect(&slots.slot_table_view_old_versions_context_menu_enabler); }
                                unsafe { (stuff.table_view_old_versions as *mut Widget).as_ref().unwrap().signals().custom_context_menu_requested().connect(&slots.slot_table_view_old_versions_context_menu); }
                                unsafe { stuff.table_view_old_versions_context_menu_load.as_mut().unwrap().signals().triggered().connect(&slots.slot_table_view_old_versions_context_menu_load); }
                                unsafe { stuff.table_view_old_versions_context_menu_delete.as_mut().unwrap().signals().triggered().connect(&slots.slot_table_view_old_versions_context_menu_delete); }

                                // Return the slots and the font.
                                Ok((slots, monospace_font))
                            }

                            // If there is no schema, return error.
                            None => return Err(ErrorKind::SchemaNotFound)?
                        }
                    },

                    // If it fails, return error.
                    Err(error) => return Err(error)
                }
            }

            // Otherwise, return error.
            else { return Err(ErrorKind::DBTableIsNotADBTable)? }
        }

        // Otherwise, return error.
        else { return Err(ErrorKind::DBTableIsNotADBTable)? }
    }

    /// This function is meant to load the static data of a DB PackedFile into the decoder, or return error.
    pub fn load_data_to_decoder_view(
        stuff: &PackedFileDBDecoderStuff,
        stuff_non_ui: &PackedFileDBDecoderStuffNonUI,
    ) {
        // Get the FontMetrics for the size stuff.
        let font;
        unsafe { font = stuff.hex_view_index.as_mut().unwrap().document().as_mut().unwrap().default_font(); }
        let font_metrics = FontMetrics::new(&font);

        // We don't need the entire PackedFile, just his begining. Otherwise, this function takes ages to finish.
        let data_reduced = if stuff_non_ui.packed_file_data.len() > 16 * 60 { &stuff_non_ui.packed_file_data[..16 * 60] }
        else { &stuff_non_ui.packed_file_data };

        // This creates the "index" column at the left of the hex data. The logic behind this, because
        // even I have problems to understand it: lines are 4 packs of 4 bytes => 16 bytes. Amount of
        // lines is "bytes we have / 16 + 1" (+ 1 because we want to show incomplete lines too).
        // Then, for the zeroes, we default to 4.
        let mut hex_lines_text = String::new();
        let hex_lines_amount = (data_reduced.len() / 16) + 1;
        for hex_line in 0..hex_lines_amount { hex_lines_text.push_str(&format!("{:>0count$X}\n", hex_line * 16, count = 4)); }

        // Add all the "Index" lines to the TextEdit.
        unsafe { stuff.hex_view_index.as_mut().unwrap().set_html(&QString::from_std_str(&hex_lines_text)); }

        // Resize the TextEdit.
        let text_size = font_metrics.size((0, &QString::from_std_str(hex_lines_text)));
        unsafe { stuff.hex_view_index.as_mut().unwrap().set_fixed_width(text_size.width() + 34); }

        // This gets the hex data into place. In big files, this takes ages, so we cut them if they
        // are longer than 100 lines to speed up loading and fix a weird issue with big tables.
        let mut hex_raw_data = format!("{:02X?}", data_reduced);

        // Remove the first and last chars.
        hex_raw_data.remove(0);
        hex_raw_data.pop();

        // Remove all the kebab, or the commas. Whatever float your boat...
        hex_raw_data.retain(|c| c != ',');

        // `hex_view_raw` TextEdit.
        {
            // Create the String to pass to the TextEdit.
            let mut hex_view_raw_string = String::new();

            // This pushes a newline after 48 characters (2 for every byte + 1 whitespace).
            for (j, i) in hex_raw_data.chars().enumerate() {

                // Also. replace the last whitespace of each line with a \n.
                if j % 48 == 0 && j != 0 { hex_view_raw_string.pop(); hex_view_raw_string.push_str("\n"); }
                hex_view_raw_string.push(i);
            }

            // Add all the "Raw" lines to the TextEdit.
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text(&QString::from_std_str(&hex_view_raw_string)); }

            // Resize the TextEdit.
            let text_size = font_metrics.size((0, &QString::from_std_str(hex_view_raw_string)));
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_fixed_width(text_size.width() + 34); }

            // Prepare the format for the header.
            let mut header_format = TextCharFormat::new();
            header_format.set_background(&Brush::new(if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkRed } else { GlobalColor::Red }));

            // Get the cursor.
            let mut cursor;
            unsafe { cursor = stuff.hex_view_raw.as_mut().unwrap().text_cursor(); }

            // Create the "Selection" for the header.
            cursor.move_position(MoveOperation::Start);
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Keep, (stuff_non_ui.initial_index * 3) as i32));

            // Block the signals during this, so we don't mess things up.
            let mut blocker;
            unsafe { blocker = SignalBlocker::new(stuff.hex_view_raw.as_mut().unwrap().static_cast_mut() as &mut Object); }

            // Set the cursor and his format.
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor); }
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_current_char_format(&header_format); }

            // Clear the selection.
            cursor.clear_selection();
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor); }

            // Unblock the signals.
            blocker.unblock();
        }

        // `hex_view_decoded` TextView.
        {
            // Create the String to pass to the TextEdit.
            let mut hex_view_decoded_string = String::new();

            // This pushes a newline after 16 characters.
            for (j, i) in data_reduced.iter().enumerate() {
                if j % 16 == 0 && j != 0 { hex_view_decoded_string.push_str("\n"); }
                let character = *i as char;

                // If is a valid UTF-8 char, show it. Otherwise, default to '.'.
                if character.is_alphanumeric() { hex_view_decoded_string.push(character); }
                else { hex_view_decoded_string.push('.'); }
            }

            // Add all the "Decoded" lines to the TextEdit.
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text(&QString::from_std_str(&hex_view_decoded_string)); }

            // Resize the TextEdit.
            let text_size = font_metrics.size((0, &QString::from_std_str(hex_view_decoded_string)));
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_fixed_width(text_size.width() + 34); }

            // Prepare the format for the header.
            let mut header_format = TextCharFormat::new();
            header_format.set_background(&Brush::new(if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkRed } else { GlobalColor::Red }));

            // Get the cursor.
            let mut cursor;
            unsafe { cursor = stuff.hex_view_decoded.as_mut().unwrap().text_cursor(); }

            // Create the "Selection" for the header. We need to add 1 char per line to this.
            cursor.move_position(MoveOperation::Start);
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Keep, (stuff_non_ui.initial_index + (stuff_non_ui.initial_index as f32 / 16.0).floor() as usize) as i32));

            // Block the signals during this, so we don't mess things up.
            let mut blocker;
            unsafe { blocker = SignalBlocker::new(stuff.hex_view_decoded.as_mut().unwrap().static_cast_mut() as &mut Object); }

            // Set the cursor and his format.
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor); }
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_current_char_format(&header_format); }

            // Clear the selection.
            cursor.clear_selection();
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor); }

            // Unblock the signals.
            blocker.unblock();
        }

        // Load the "Info" data to the view.
        unsafe { stuff.table_info_type_decoded_label.as_mut().unwrap().set_text(&QString::from_std_str(&stuff_non_ui.packed_file_path[1])); }
        unsafe { stuff.table_info_version_decoded_label.as_mut().unwrap().set_text(&QString::from_std_str(format!("{}", stuff_non_ui.version))); }
        unsafe { stuff.table_info_entry_count_decoded_label.as_mut().unwrap().set_text(&QString::from_std_str(format!("{}", stuff_non_ui.entry_count))); }
    }

    /// This function is used to update the list of "Versions" of the currently open table decoded.
    pub fn update_versions_list(
        stuff: &PackedFileDBDecoderStuff,
        schema: &Schema,
        table_name: &str,
    ) {
        // Clear the current list.
        unsafe { stuff.table_model_old_versions.as_mut().unwrap().clear(); }

        // And get all the versions of this table, and list them in their TreeView, if we have any.
        if let Some(table_versions_list) = DB::get_schema_versions_list(table_name, schema) {
            for version in table_versions_list {
                let item = StandardItem::new(&QString::from_std_str(format!("{}", version.version)));
                unsafe { stuff.table_model_old_versions.as_mut().unwrap().append_row_unsafe(item.into_raw()); }
            }
        }

        // Set the title of the column.
        unsafe { stuff.table_model_old_versions.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Versions Decoded")))); }

        // Extend the column to fill the Table.
        unsafe { stuff.table_view_old_versions.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_section_resize_mode(ResizeMode::Stretch); }
    }

    /// This function updates the data shown in the decoder view when we execute it. `index_data`
    /// is the position from where to start decoding. In field_list the boolean is true for the first load.
    /// Otherwise, always pass false there.
    pub fn update_decoder_view(
        stuff: &PackedFileDBDecoderStuff,
        stuff_non_ui: &PackedFileDBDecoderStuffNonUI,
        field_list: (bool, &[Field]),
        mut index_data: &mut usize,
    ) {

        // Create the variables to hold the values we'll pass to the LineEdits.
        let decoded_bool;
        let decoded_float;
        let decoded_integer;
        let decoded_long_integer;
        let decoded_string_u8;
        let decoded_string_u16;
        let decoded_optional_string_u8;
        let decoded_optional_string_u16;

        // If we are loading data to the table for the first time, we'll load to the table all the data
        // directly from the existing definition and update the initial index for decoding.
        if field_list.0 {
            for field in field_list.1 {
                Self::add_field_to_data_view(
                    &stuff,
                    &stuff_non_ui,
                    &field.field_name,
                    field.field_type.to_owned(),
                    field.field_is_key,
                    &field.field_is_reference,
                    &field.field_description,
                    &mut index_data,
                );
            }
        }

        // Check if the index does even exist, to avoid crashes.
        if stuff_non_ui.packed_file_data.get(*index_data).is_some() {
            decoded_bool = match coding_helpers::decode_packedfile_bool(stuff_non_ui.packed_file_data[*index_data], &mut index_data.clone()) {
                Ok(data) => if data { "True" } else { "False" },
                Err(_) => "Error"
            };

            decoded_optional_string_u8 = match coding_helpers::decode_packedfile_optional_string_u8(&stuff_non_ui.packed_file_data[*index_data..], &mut index_data.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };

            decoded_optional_string_u16 = match coding_helpers::decode_packedfile_optional_string_u16(&stuff_non_ui.packed_file_data[*index_data..], &mut index_data.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_bool = "Error";
            decoded_optional_string_u8 = "Error".to_owned();
            decoded_optional_string_u16 = "Error".to_owned();
        };

        // Check if the index does even exist, to avoid crashes.
        if stuff_non_ui.packed_file_data.get(*index_data + 3).is_some() {
            decoded_float = match coding_helpers::decode_packedfile_float_f32(&stuff_non_ui.packed_file_data[*index_data..(*index_data + 4)], &mut index_data.clone()) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned(),
            };

            decoded_integer = match coding_helpers::decode_packedfile_integer_i32(&stuff_non_ui.packed_file_data[*index_data..(*index_data + 4)], &mut index_data.clone()) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_float = "Error".to_owned();
            decoded_integer = "Error".to_owned();
        }

        // Check if the index does even exist, to avoid crashes.
        decoded_long_integer = if stuff_non_ui.packed_file_data.get(*index_data + 7).is_some() {
            match coding_helpers::decode_packedfile_integer_i64(&stuff_non_ui.packed_file_data[*index_data..(*index_data + 8)], &mut index_data.clone()) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned()
            }
        }
        else { "Error".to_owned() };

        // Check that the index exist, to avoid crashes.
        if stuff_non_ui.packed_file_data.get(*index_data + 1).is_some() {
            decoded_string_u8 = match coding_helpers::decode_packedfile_string_u8(&stuff_non_ui.packed_file_data[*index_data..], &mut index_data.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };

            decoded_string_u16 = match coding_helpers::decode_packedfile_string_u16(&stuff_non_ui.packed_file_data[*index_data..], &mut index_data.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_string_u8 = "Error".to_owned();
            decoded_string_u16 = "Error".to_owned();
        }

        // We update all the decoded entries here.
        unsafe { stuff.bool_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_bool)); }
        unsafe { stuff.float_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_float)); }
        unsafe { stuff.integer_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_integer)); }
        unsafe { stuff.long_integer_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_long_integer)); }
        unsafe { stuff.string_u8_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_string_u8))); }
        unsafe { stuff.string_u16_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_string_u16))); }
        unsafe { stuff.optional_string_u8_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_optional_string_u8))); }
        unsafe { stuff.optional_string_u16_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_optional_string_u16))); }

        // Prepare the format for the cleaning.
        let mut neutral_format = TextCharFormat::new();
        neutral_format.set_background(&Brush::new(GlobalColor::Transparent));

        // Prepare the format for the decoded row.
        let mut decoded_format = TextCharFormat::new();
        decoded_format.set_background(&Brush::new(if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkYellow } else { GlobalColor::Yellow }));

        // Prepare the format for the current index.
        let mut index_format = TextCharFormat::new();
        index_format.set_background(&Brush::new(if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::DarkMagenta } else { GlobalColor::Magenta }));

        // Clean both TextEdits.
        {

            // Get the cursor.
            let mut cursor;
            unsafe { cursor = stuff.hex_view_raw.as_mut().unwrap().text_cursor(); }

            // Create the "Selection" for the rest of the data.
            cursor.move_position(MoveOperation::Start);
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Move, (stuff_non_ui.initial_index * 3) as i32));
            cursor.move_position((MoveOperation::End, MoveMode::Keep));

            // Block the signals during this, so we don't mess things up.
            let mut blocker;
            unsafe { blocker = SignalBlocker::new(stuff.hex_view_raw.as_mut().unwrap().static_cast_mut() as &mut Object); }

            // Set the cursor and his format.
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor); }
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_current_char_format(&neutral_format); }

            // Clear the selection.
            cursor.clear_selection();
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor); }

            // Unblock the signals.
            blocker.unblock();

            // Get the cursor.
            let mut cursor;
            unsafe { cursor = stuff.hex_view_decoded.as_mut().unwrap().text_cursor(); }

            // Create the "Selection" for the rest of the data.
            cursor.move_position(MoveOperation::Start);
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Move, (stuff_non_ui.initial_index + (stuff_non_ui.initial_index as f32 / 16.0).floor() as usize) as i32));
            cursor.move_position((MoveOperation::End, MoveMode::Keep));

            // Block the signals during this, so we don't mess things up.
            let mut blocker;
            unsafe { blocker = SignalBlocker::new(stuff.hex_view_decoded.as_mut().unwrap().static_cast_mut() as &mut Object); }

            // Set the cursor and his format.
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor); }
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_current_char_format(&neutral_format); }

            // Clear the selection.
            cursor.clear_selection();
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor); }

            // Unblock the signals.
            blocker.unblock();
        }

        // Paint both decoded rows.
        {

            // Get the cursor.
            let mut cursor;
            unsafe { cursor = stuff.hex_view_raw.as_mut().unwrap().text_cursor(); }

            // Create the "Selection" for the decoded row.
            cursor.move_position(MoveOperation::Start);
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Move, (stuff_non_ui.initial_index * 3) as i32));
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Keep, ((*index_data - stuff_non_ui.initial_index) * 3) as i32));

            // Block the signals during this, so we don't mess things up.
            let mut blocker;
            unsafe { blocker = SignalBlocker::new(stuff.hex_view_raw.as_mut().unwrap().static_cast_mut() as &mut Object); }

            // Set the cursor and his format.
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor); }
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_current_char_format(&decoded_format); }

            // Clear the selection.
            cursor.clear_selection();
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor); }

            // Unblock the signals.
            blocker.unblock();

            // Get the cursor.
            let mut cursor;
            unsafe { cursor = stuff.hex_view_decoded.as_mut().unwrap().text_cursor(); }

            // Create the "Selection" for the decoded row.
            let positions_to_move_end = *index_data / 16;
            let positions_to_move_start = stuff_non_ui.initial_index / 16;
            let positions_to_move_vertical = positions_to_move_end - positions_to_move_start;
            let positions_to_move_horizontal = *index_data - stuff_non_ui.initial_index;
            let positions_to_move = positions_to_move_horizontal + positions_to_move_vertical;

            cursor.move_position(MoveOperation::Start);
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Move, (stuff_non_ui.initial_index + (stuff_non_ui.initial_index as f32 / 16.0).floor() as usize) as i32));
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Keep, positions_to_move as i32));

            // Block the signals during this, so we don't mess things up.
            let mut blocker;
            unsafe { blocker = SignalBlocker::new(stuff.hex_view_decoded.as_mut().unwrap().static_cast_mut() as &mut Object); }

            // Set the cursor and his format.
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor); }
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_current_char_format(&decoded_format); }

            // Clear the selection.
            cursor.clear_selection();
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor); }

            // Unblock the signals.
            blocker.unblock();
        }

        // Paint both current index.
        {

            // Get the cursor.
            let mut cursor;
            unsafe { cursor = stuff.hex_view_raw.as_mut().unwrap().text_cursor(); }

            // Create the "Selection" for the decoded row.
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Keep, 3));

            // Block the signals during this, so we don't mess things up.
            let mut blocker;
            unsafe { blocker = SignalBlocker::new(stuff.hex_view_raw.as_mut().unwrap().static_cast_mut() as &mut Object); }

            // Set the cursor and his format.
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor); }
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_current_char_format(&index_format); }

            // Clear the selection.
            cursor.clear_selection();
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor); }

            // Unblock the signals.
            blocker.unblock();

            // Get the cursor.
            let mut cursor;
            unsafe { cursor = stuff.hex_view_decoded.as_mut().unwrap().text_cursor(); }

            // Create the "Selection" for the decoded row.
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Keep, 1));

            // Block the signals during this, so we don't mess things up.
            let mut blocker;
            unsafe { blocker = SignalBlocker::new(stuff.hex_view_decoded.as_mut().unwrap().static_cast_mut() as &mut Object); }

            // Set the cursor and his format.
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor); }
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_current_char_format(&index_format); }

            // Clear the selection.
            cursor.clear_selection();
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor); }

            // Unblock the signals.
            blocker.unblock();
        }
    }

    /// This function adds fields to the decoder's table, so we can do this without depending on the
    /// updates of the decoder's view. As this has a lot of required data, lets's explain the weirdest ones:
    /// - index_data: the index to start decoding from the vector.
    /// - index_row: the position in the row. None to calculate the last position's number.
    pub fn add_field_to_data_view(
        stuff: &PackedFileDBDecoderStuff,
        stuff_non_ui: &PackedFileDBDecoderStuffNonUI,
        field_name: &str,
        field_type: FieldType,
        field_is_key: bool,
        field_is_reference: &Option<(String, String)>,
        field_description: &str,
        mut index_data: &mut usize,
    ) {

        // Decode the data from the field.
        let decoded_data = Self::decode_data_by_fieldtype(
            &stuff_non_ui.packed_file_data,
            &field_type,
            &mut index_data
        );

        // Get the type of the data we are going to put into the Table.
        let field_type = match field_type {
            FieldType::Boolean => "Bool",
            FieldType::Float => "Float",
            FieldType::Integer => "Integer",
            FieldType::LongInteger => "LongInteger",
            FieldType::StringU8 => "StringU8",
            FieldType::StringU16 => "StringU16",
            FieldType::OptionalStringU8 => "OptionalStringU8",
            FieldType::OptionalStringU16 => "OptionalStringU16",
        };

        // If the field has a reference...
        if let Some(ref reference) = *field_is_reference {

            // Create a new list of StandardItem.
            let mut qlist = ListStandardItemMutPtr::new(());

            // Create the items of the new row.
            let field_name = StandardItem::new(&QString::from_std_str(field_name));
            let field_type = StandardItem::new(&QString::from_std_str(field_type));
            let mut field_is_key_item = StandardItem::new(());
            field_is_key_item.set_editable(false);
            field_is_key_item.set_checkable(true);
            field_is_key_item.set_check_state(if field_is_key { CheckState::Checked } else { CheckState::Unchecked });
            let reference_table = StandardItem::new(&QString::from_std_str(&reference.0));
            let reference_field = StandardItem::new(&QString::from_std_str(&reference.1));
            let mut decoded_data = StandardItem::new(&QString::from_std_str(&decoded_data));
            let field_description = StandardItem::new(&QString::from_std_str(field_description));

            // The "Decoded First Row" column should not be editable.
            decoded_data.set_editable(false);

            // Add the items to the list.
            unsafe { qlist.append_unsafe(&field_name.into_raw()); }
            unsafe { qlist.append_unsafe(&field_type.into_raw()); }
            unsafe { qlist.append_unsafe(&field_is_key_item.into_raw()); }
            unsafe { qlist.append_unsafe(&reference_table.into_raw()); }
            unsafe { qlist.append_unsafe(&reference_field.into_raw()); }
            unsafe { qlist.append_unsafe(&decoded_data.into_raw()); }
            unsafe { qlist.append_unsafe(&field_description.into_raw()); }

            // Just append a new row.
            unsafe { stuff.table_model.as_mut().unwrap().append_row(&qlist); }
        }

        // Otherwise, we pass an empty reference.
        else {

            // Create a new list of StandardItem.
            let mut qlist = ListStandardItemMutPtr::new(());

            // Create the items of the new row.
            let field_name = StandardItem::new(&QString::from_std_str(field_name));
            let field_type = StandardItem::new(&QString::from_std_str(field_type));
            let mut field_is_key_item = StandardItem::new(());
            field_is_key_item.set_editable(false);
            field_is_key_item.set_checkable(true);
            field_is_key_item.set_check_state(if field_is_key { CheckState::Checked } else { CheckState::Unchecked });
            let reference_table = StandardItem::new(&QString::from_std_str(""));
            let reference_field = StandardItem::new(&QString::from_std_str(""));
            let mut decoded_data = StandardItem::new(&QString::from_std_str(&decoded_data));
            let field_description = StandardItem::new(&QString::from_std_str(field_description));

            // The "Decoded First Row" column should not be editable.
            decoded_data.set_editable(false);

            // Add the items to the list.
            unsafe { qlist.append_unsafe(&field_name.into_raw()); }
            unsafe { qlist.append_unsafe(&field_type.into_raw()); }
            unsafe { qlist.append_unsafe(&field_is_key_item.into_raw()); }
            unsafe { qlist.append_unsafe(&reference_table.into_raw()); }
            unsafe { qlist.append_unsafe(&reference_field.into_raw()); }
            unsafe { qlist.append_unsafe(&decoded_data.into_raw()); }
            unsafe { qlist.append_unsafe(&field_description.into_raw()); }

            // Just append a new row.
            unsafe { stuff.table_model.as_mut().unwrap().append_row(&qlist); }
        }

        // Set the title of the columns and extend them, just in case is needed.
        unsafe { stuff.table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
        unsafe { stuff.table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Field Name")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Field Type")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((2, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Is key?")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((3, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Ref. to Table")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((4, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Ref. to Column")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((5, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("First Row Decoded")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((6, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Description")))); }

        // The second field should be a combobox.
        let mut list = StringList::new(());
        list.append(&QString::from_std_str("Bool"));
        list.append(&QString::from_std_str("Float"));
        list.append(&QString::from_std_str("Integer"));
        list.append(&QString::from_std_str("LongInteger"));
        list.append(&QString::from_std_str("StringU8"));
        list.append(&QString::from_std_str("StringU16"));
        list.append(&QString::from_std_str("OptionalStringU8"));
        list.append(&QString::from_std_str("OptionalStringU16"));
        let list: *mut StringList = &mut list;
        unsafe { qt_custom_stuff::new_combobox_item_delegate(stuff.table_view as *mut Object, 1, list as *const StringList, false)};
    }

    /// This function is a helper to try to decode data in different formats, returning "Error" in case
    /// of decoding error. It requires the FieldType we want to decode, the data we want to decode
    /// (vec<u8>, being the first u8 the first byte to decode) and the index of the data in the Vec<u8>.
    fn decode_data_by_fieldtype(
        field_data: &[u8],
        field_type: &FieldType,
        mut index_data: &mut usize
    ) -> String {

        // Try to decode the field, depending on what type that field is.
        match *field_type {

            // If the field is a boolean...
            FieldType::Boolean => {

                // Check if the index does even exist, to avoid crashes.
                if field_data.get(*index_data).is_some() {
                    match coding_helpers::decode_packedfile_bool(field_data[*index_data], &mut index_data) {
                        Ok(result) => {
                            if result { "True".to_string() }
                            else { "False".to_string() }
                        }
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::Float => {
                if field_data.get(*index_data + 3).is_some() {
                    match coding_helpers::decode_packedfile_float_f32(&field_data[*index_data..(*index_data + 4)], &mut index_data) {
                        Ok(result) => result.to_string(),
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::Integer => {
                if field_data.get(*index_data + 3).is_some() {
                    match coding_helpers::decode_packedfile_integer_i32(&field_data[*index_data..(*index_data + 4)], &mut index_data) {
                        Ok(result) => result.to_string(),
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::LongInteger => {
                if field_data.get(*index_data + 7).is_some() {
                    match coding_helpers::decode_packedfile_integer_i64(&field_data[*index_data..(*index_data + 8)], &mut index_data) {
                        Ok(result) => result.to_string(),
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::StringU8 => {
                if field_data.get(*index_data + 1).is_some() {
                    match coding_helpers::decode_packedfile_string_u8(&field_data[*index_data..], &mut index_data) {
                        Ok(result) => result,
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::StringU16 => {
                if field_data.get(*index_data + 1).is_some() {
                    match coding_helpers::decode_packedfile_string_u16(&field_data[*index_data..], &mut index_data) {
                        Ok(result) => result,
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::OptionalStringU8 => {
                if field_data.get(*index_data).is_some() {
                    match coding_helpers::decode_packedfile_optional_string_u8(&field_data[*index_data..], &mut index_data) {
                        Ok(result) => result,
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::OptionalStringU16 => {
                if field_data.get(*index_data).is_some() {
                    match coding_helpers::decode_packedfile_optional_string_u16(&field_data[*index_data..], &mut index_data) {
                        Ok(result) => result,
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
        }
    }

    /// This function is used to update the decoder view when we try to add a new field to
    /// the schema with one of the "Use this" buttons.
    pub fn use_this(
        stuff: &PackedFileDBDecoderStuff,
        stuff_non_ui: &PackedFileDBDecoderStuffNonUI,
        field_type: FieldType,
        mut index_data: &mut usize,
    ) {

        // Try to add the field, and update the index with it.
        Self::add_field_to_data_view(
            &stuff,
            &stuff_non_ui,
            "new_field",
            field_type,
            false,
            &None,
            "",
            &mut index_data,
        );

        // Update all the dynamic data of the "Decoder" view.
        Self::update_decoder_view(
            &stuff,
            &stuff_non_ui,
            (false, &[]),
            &mut index_data,
        );
    }

    /// This function updates the "First row decoded" column in the decoder view, the current index and
    /// the decoded entries. This should be called in row changes (deletion and moving, not adding).
    fn update_first_row_decoded(
        stuff: &PackedFileDBDecoderStuff,
        stuff_non_ui: &PackedFileDBDecoderStuffNonUI,
        mut index: &mut usize,
    ) -> Vec<i32> {

        // Create the list of "invalid" types.
        let mut invalid_types = vec![];

        // Reset the index.
        *index = stuff_non_ui.initial_index;

        // Get the first row.
        let mut row = 0;

        // Loop through all the rows.
        loop {

            // Get the ModelIndex of the cell we want to update.
            let model_index;
            unsafe { model_index = stuff.table_model.as_mut().unwrap().index((row, 5)); }

            // If it's valid (exists)...
            if model_index.is_valid() {

                // Get the row's type.
                let row_type;
                unsafe { row_type = stuff.table_model.as_mut().unwrap().index((row, 1)); }

                // Get the field's type.
                let field_type = match &*row_type.data(0).to_string().to_std_string() {
                    "Bool" => FieldType::Boolean,
                    "Float" => FieldType::Float,
                    "Integer" => FieldType::Integer,
                    "LongInteger" => FieldType::LongInteger,
                    "StringU8" => FieldType::StringU8,
                    "StringU16" => FieldType::StringU16,
                    "OptionalStringU8" => FieldType::OptionalStringU8,
                    "OptionalStringU16" => FieldType::OptionalStringU16,

                    // In case of invalid type, we add it to the list and set it as bool.
                    _ => {

                        // Add the row to the list.
                        invalid_types.push(row);

                        // Return a boolean.
                        FieldType::Boolean
                    }
                };

                // Get the decoded data using it's type...
                let decoded_data = Self::decode_data_by_fieldtype(
                    &stuff_non_ui.packed_file_data,
                    &field_type,
                    &mut index
                );

                // Get the item from the "First Row Decoded" column.
                let item;
                unsafe { item = stuff.table_model.as_mut().unwrap().item_from_index(&model_index); }

                // Change it to our decoded data.
                unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str(decoded_data)); }

                // Increase the row.
                row += 1;
            }

            // Otherwise, stop the loop.
            else { break; }
        }

        // Update the entire decoder to use the new index.
        Self::update_decoder_view(&stuff, &stuff_non_ui, (false, &[]), &mut index);

        // Return the list of broken rows.
        invalid_types
    }

    /// This function gets the data from the decoder's table, and returns it, so we can save it in a TableDefinition.
    pub fn return_data_from_data_view(
        stuff: &PackedFileDBDecoderStuff
    ) -> Vec<Field> {

        // Create the field's vector.
        let mut fields = vec![];

        // Get the first row.
        let mut row = 0;

        // Loop through all the rows.
        loop {

            // Get a ModelIndex from the row.
            let model_index;
            unsafe { model_index = stuff.table_model.as_mut().unwrap().index((row, 0)); }

            // If it's valid (exists)...
            if model_index.is_valid() {

                // Get the data from each field of the row...
                let field_name;
                let field_type;
                let field_is_key;
                let ref_table;
                let ref_column;
                let field_description;

                unsafe { field_name = stuff.table_model.as_mut().unwrap().item((row, 0)).as_mut().unwrap().text().to_std_string(); }
                unsafe { field_type = stuff.table_model.as_mut().unwrap().item((row, 1)).as_mut().unwrap().text().to_std_string(); }
                unsafe { field_is_key = if stuff.table_model.as_mut().unwrap().item((row, 2)).as_mut().unwrap().check_state() == CheckState::Checked { true } else { false }; }
                unsafe { ref_table = stuff.table_model.as_mut().unwrap().item((row, 3)).as_mut().unwrap().text().to_std_string(); }
                unsafe { ref_column = stuff.table_model.as_mut().unwrap().item((row, 4)).as_mut().unwrap().text().to_std_string(); }
                unsafe { field_description = stuff.table_model.as_mut().unwrap().item((row, 6)).as_mut().unwrap().text().to_std_string(); }

                // Get the proper type of the field. If invalid, default to OptionalStringU16.
                let field_type = match &*field_type {
                    "Bool" => FieldType::Boolean,
                    "Float" => FieldType::Float,
                    "Integer" => FieldType::Integer,
                    "LongInteger" => FieldType::LongInteger,
                    "StringU8" => FieldType::StringU8,
                    "StringU16" => FieldType::StringU16,
                    "OptionalStringU8" => FieldType::OptionalStringU8,
                    "OptionalStringU16" | _=> FieldType::OptionalStringU16,
                };

                // If there is no table referenced...
                if ref_table.is_empty() { fields.push(Field::new(field_name, field_type, field_is_key, None, field_description)); }

                // Otherwise...
                else { fields.push(Field::new(field_name, field_type, field_is_key, Some((ref_table, ref_column)), field_description)); }

                // Increase the row.
                row += 1;
            }

            // Otherwise, stop the loop.
            else { break; }
        }

        // Return the fields.
        fields
    }

    /// This function updates the "selection" fields when the selection of a HexView is changed.
    fn update_selection_decoded_fields(
        stuff: &PackedFileDBDecoderStuff,
        stuff_non_ui: &PackedFileDBDecoderStuffNonUI,
        selection_start: usize
    ) {

        // Create the variables to hold the values we'll pass to the LineEdits.
        let decoded_bool;
        let decoded_float;
        let decoded_integer;
        let decoded_long_integer;
        let decoded_string_u8;
        let decoded_string_u16;
        let decoded_optional_string_u8;
        let decoded_optional_string_u16;

        // Check if the index does even exist, to avoid crashes.
        if stuff_non_ui.packed_file_data.get(selection_start).is_some() {
            decoded_bool = match coding_helpers::decode_packedfile_bool(stuff_non_ui.packed_file_data[selection_start], &mut selection_start.clone()) {
                Ok(data) => if data { "True" } else { "False" },
                Err(_) => "Error"
            };

            decoded_optional_string_u8 = match coding_helpers::decode_packedfile_optional_string_u8(&stuff_non_ui.packed_file_data[selection_start..], &mut selection_start.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };

            decoded_optional_string_u16 = match coding_helpers::decode_packedfile_optional_string_u16(&stuff_non_ui.packed_file_data[selection_start..], &mut selection_start.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_bool = "Error";
            decoded_optional_string_u8 = "Error".to_owned();
            decoded_optional_string_u16 = "Error".to_owned();
        };

        // Check if the index does even exist, to avoid crashes.
        if stuff_non_ui.packed_file_data.get(selection_start + 3).is_some() {
            decoded_float = match coding_helpers::decode_packedfile_float_f32(&stuff_non_ui.packed_file_data[selection_start..(selection_start + 4)], &mut selection_start.clone()) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned(),
            };

            decoded_integer = match coding_helpers::decode_packedfile_integer_i32(&stuff_non_ui.packed_file_data[selection_start..(selection_start + 4)], &mut selection_start.clone()) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_float = "Error".to_owned();
            decoded_integer = "Error".to_owned();
        }

        // Check if the index does even exist, to avoid crashes.
        decoded_long_integer = if stuff_non_ui.packed_file_data.get(selection_start + 7).is_some() {
            match coding_helpers::decode_packedfile_integer_i64(&stuff_non_ui.packed_file_data[selection_start..(selection_start + 8)], &mut selection_start.clone()) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned()
            }
        }
        else { "Error".to_owned() };

        // Check that the index exist, to avoid crashes.
        if stuff_non_ui.packed_file_data.get(selection_start + 1).is_some() {
            decoded_string_u8 = match coding_helpers::decode_packedfile_string_u8(&stuff_non_ui.packed_file_data[selection_start..], &mut selection_start.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };

            decoded_string_u16 = match coding_helpers::decode_packedfile_string_u16(&stuff_non_ui.packed_file_data[selection_start..], &mut selection_start.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_string_u8 = "Error".to_owned();
            decoded_string_u16 = "Error".to_owned();
        }

        // We update all the decoded entries here.
        unsafe { stuff.selection_bool_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_bool)); }
        unsafe { stuff.selection_float_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_float)); }
        unsafe { stuff.selection_integer_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_integer)); }
        unsafe { stuff.selection_long_integer_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_long_integer)); }
        unsafe { stuff.selection_string_u8_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_string_u8))); }
        unsafe { stuff.selection_string_u16_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_string_u16))); }
        unsafe { stuff.selection_optional_string_u8_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_optional_string_u8))); }
        unsafe { stuff.selection_optional_string_u16_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_optional_string_u16))); }
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
/// His intended use is for just after we reload the data to the table.
fn build_columns(
    definition: &TableDefinition,
    table_view: *mut TableView,
    model: *mut StandardItemModel
) {
    // Create a list of "Key" columns.
    let mut keys = vec![];

    // For each column, clean their name and set their width and tooltip.
    for (index, field) in definition.fields.iter().enumerate() {

        let name = clean_column_names(&field.field_name);
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

        // Create the tooltip for the column.
        let tooltip_text: String = if let Some(ref reference) = field.field_is_reference {
            if !field.field_description.is_empty() { format!("{}\n\nThis column is a reference to \"{}/{}\".", field.field_description, reference.0, reference.1) }
            else { format!("This column is a reference to \"{}/{}\".", reference.0, reference.1) }
        } else { field.field_description.to_owned() };
        unsafe { item.as_mut().unwrap().set_tool_tip(&QString::from_std_str(&tooltip_text)); }
            
        // If the field is key, add that column to the "Key" list, so we can move them at the begining later.
        if field.field_is_key { keys.push(index); }
    }

    // If we have any "Key" field, move it to the beginning.
    if !keys.is_empty() {
        for (position, column) in keys.iter().enumerate() {
            unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().move_section(*column as i32, position as i32); }
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
        if ref_data.contains(&text) { unsafe { item.as_mut().unwrap().set_foreground(&Brush::new(if *SETTINGS.lock().unwrap().settings_bool.get("use_dark_theme").unwrap() { GlobalColor::White } else { GlobalColor::Black })); } }
        else if ref_data.is_empty() { unsafe { item.as_mut().unwrap().set_foreground(&Brush::new(GlobalColor::Blue)); } }
        else { unsafe { item.as_mut().unwrap().set_foreground(&Brush::new(GlobalColor::Red)); } }
    }
}

/// This function checks if the data in the clipboard is suitable for the selected Items.
fn check_clipboard(
    definition: &TableDefinition,
    table_view: *mut TableView,
    model: *mut StandardItemModel,
    filter_model: *mut SortFilterProxyModel,
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
            if header.visual_index(a.column()) < header.visual_index(b.column()) { return Ordering::Less }
            else { return Ordering::Greater }
        } else { return Ordering::Equal }
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

        // Depending on the column, we try to encode the data in one format or another.
        let column = unsafe { cell.0.as_mut().unwrap().index().column() };
        match definition.fields[column as usize].field_type {
            FieldType::Boolean =>  if cell.1.to_lowercase() != "true" && cell.1.to_lowercase() != "false" && cell.1 != "1" && cell.1 != "0" { return false },
            FieldType::Float => if cell.1.parse::<f32>().is_err() { return false },
            FieldType::Integer => if cell.1.parse::<i32>().is_err() { return false },
            FieldType::LongInteger => if cell.1.parse::<i64>().is_err() { return false },

            // All these are Strings, so we can skip their checks....
            FieldType::StringU8 |
            FieldType::StringU16 |
            FieldType::OptionalStringU8 |
            FieldType::OptionalStringU16 => continue
        }
    }

    // If we reach this place, it means none of the cells was incorrect, so we can paste.
    true
}

/// This function checks if the data in the clipboard is suitable for be pasted in all selected cells.
fn check_clipboard_to_fill_selection(
    definition: &TableDefinition,
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
    definition: &TableDefinition
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
    if let Some(state) = HISTORY_TABLE_STATE.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
        unsafe { state.filter_state = FilterState::new(filter_line_edit.as_mut().unwrap().text().to_std_string(), column_selector.as_mut().unwrap().current_index(), case_sensitive_button.as_mut().unwrap().is_checked()); }
    }
}
