//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module containing the ffi functions used for custom widgets.
!*/

use qt_widgets::QTableView;
use qt_widgets::QWidget;

use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::QAbstractItemModel;
use qt_core::QObject;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QStringList;

use cpp_core::MutPtr;
use cpp_core::Ptr;

/// This function replaces the default editor widget for reference columns with a combobox, so you can select the reference data.
extern "C" { fn new_combobox_item_delegate(table_view: *mut QObject, column: i32, list: *const QStringList, is_editable: bool, max_lenght: i32); }
pub fn new_combobox_item_delegate_safe(table_view: &mut QObject, column: i32, list: Ptr<QStringList>, is_editable: bool, max_lenght: i32) {
    unsafe { new_combobox_item_delegate(table_view, column, list.as_raw_ptr(), is_editable, max_lenght) }
}

/// This function changes the default editor widget for I32/64 cells on tables with a numeric one.
extern "C" { fn new_spinbox_item_delegate(table_view: *mut QObject, column: i32, integer_type: i32); }
pub fn new_spinbox_item_delegate_safe(table_view: &mut QObject, column: i32, integer_type: i32) {
    unsafe { new_spinbox_item_delegate(table_view, column, integer_type) }
}

/// This function changes the default editor widget for F32 cells on tables with a numeric one.
extern "C" { fn new_doublespinbox_item_delegate(table_view: *mut QObject, column: i32); }
pub fn new_doublespinbox_item_delegate_safe(table_view: &mut QObject, column: i32) {
    unsafe { new_doublespinbox_item_delegate(table_view, column) }
}

/// This function changes the default editor widget for String cells, to ensure the provided data is valid for the schema..
extern "C" { fn new_qstring_item_delegate(table_view: *mut QObject, column: i32, max_lenght: i32); }
pub fn new_qstring_item_delegate_safe(table_view: &mut QObject, column: i32, max_lenght: i32) {
    unsafe { new_qstring_item_delegate(table_view, column, max_lenght) }
}

/// This function setup the special filter used for the PackFile Contents `TreeView`.
extern "C" { fn new_treeview_filter(parent: *mut QObject) -> *mut QSortFilterProxyModel; }
pub fn new_treeview_filter_safe(parent: &mut QObject) -> MutPtr<QSortFilterProxyModel> {
    unsafe { MutPtr::from_raw(new_treeview_filter(parent)) }
}

/// This function triggers the special filter used for the PackFile Contents `TreeView`. It has to be triggered here to work properly.
extern "C" { fn trigger_treeview_filter(filter: *mut QSortFilterProxyModel, pattern: *mut QRegExp); }
pub fn trigger_treeview_filter_safe(filter: &mut QSortFilterProxyModel, pattern: &mut QRegExp) {
    unsafe { trigger_treeview_filter(filter, pattern); }
}

/// This function allows you to create a table capable of freezing columns.
extern "C" { fn new_tableview_frozen(model: *mut QAbstractItemModel, frozen_table: *mut QTableView) -> *mut QTableView; }
pub fn new_tableview_frozen_safe(model: &mut QAbstractItemModel, frozen_table: &mut QTableView) -> MutPtr<QTableView> {
    unsafe { MutPtr::from_raw(new_tableview_frozen(model, frozen_table)) }
}

/// This function allow us to create a model compatible with draggable items
extern "C" { fn new_packed_file_model() -> *mut QStandardItemModel; }
pub fn new_packed_file_model_safe() -> MutPtr<QStandardItemModel> {
    unsafe { MutPtr::from_raw(new_packed_file_model()) }
}

/// This function allow us to create a properly sized TableView for the Command Palette.
extern "C" { fn new_tableview_command_palette() -> *mut QTableView; }
pub fn new_tableview_command_palette_safe() -> MutPtr<QTableView> {
    unsafe { MutPtr::from_raw(new_tableview_command_palette()) }
}

/// This function allow us to append items to QListOfQStandardItem.
///
/// TODO: Remove when the related bug in the qt bindings gets fixed.
extern "C" { fn add_to_q_list(list: *mut QListOfQStandardItem, item: *mut QStandardItem); }
pub fn add_to_q_list_safe(list: MutPtr<QListOfQStandardItem>, item: MutPtr<QStandardItem>) {
    unsafe { add_to_q_list(list.as_mut_raw_ptr(), item.as_mut_raw_ptr()) }
}

//---------------------------------------------------------------------------//
// KTextEditor stuff.
//---------------------------------------------------------------------------//

/// This function allow us to create a complete KTextEditor.
extern "C" { fn new_text_editor(parent: *mut QWidget) -> *mut QWidget; }
pub fn new_text_editor_safe(parent: &mut QWidget) -> MutPtr<QWidget> {
    unsafe { MutPtr::from_raw(new_text_editor(parent)) }
}

/// This function allow us to get the text from the provided KTextEditor.
extern "C" { fn get_text(document: *mut QWidget) -> *mut QString; }
pub fn get_text_safe(document: &mut QWidget) -> MutPtr<QString> {
    unsafe { MutPtr::from_raw(get_text(document)) }
}

/// This function allow us to set the text of  the provided KTextEditor.
extern "C" { fn set_text(document: *mut QWidget, string: *mut QString, highlighting_mode: *mut QString); }
pub fn set_text_safe(document: &mut QWidget, string: &mut QString, highlighting_mode: &mut QString) {
    unsafe { set_text(document, string, highlighting_mode) }
}

/// This function triggers the config dialog for the KTextEditor.
extern "C" { fn open_text_editor_config(parent: *mut QWidget); }
pub fn open_text_editor_config_safe(parent: &mut QWidget) {
    unsafe { open_text_editor_config(parent) }
}
