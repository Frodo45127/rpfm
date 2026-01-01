//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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

use qt_widgets::QAbstractSpinBox;
use qt_widgets::QAction;
#[cfg(feature = "enable_tools")] use qt_widgets::QDialog;
use qt_widgets::QLabel;
use qt_widgets::QLayout;
use qt_widgets::QLineEdit;
use qt_widgets::QMainWindow;
use qt_widgets::{QMessageBox, q_message_box};
use qt_widgets::QTableView;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

#[cfg(feature = "enable_tools")] use qt_gui::QColor;
use qt_gui::QPixmap;
use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QBuffer;
use qt_core::QByteArray;
#[cfg(feature = "support_model_renderer")] use qt_core::QListOfQByteArray;
use qt_core::QListOfQObject;
#[cfg(feature = "support_model_renderer")] use qt_core::QListOfQString;
use qt_core::QModelIndex;
use qt_core::QObject;
use qt_core::QPoint;
use qt_core::QRegExp;
use qt_core::Signal;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QStringList;
use qt_core::QPtr;
use qt_core::QTimer;
use qt_core::QListOfInt;
use qt_core::QVariant;
use qt_core::CaseSensitivity;

#[cfg(feature = "enable_tools")] use cpp_core::CppBox;
use cpp_core::Ptr;

#[cfg(feature = "support_model_renderer")] use anyhow::{anyhow, Result};

#[cfg(feature = "support_model_renderer")] use std::collections::HashMap;

#[cfg(feature = "support_model_renderer")] use rpfm_lib::integrations::log;
#[cfg(feature = "support_model_renderer")] use rpfm_lib::files::ContainerPath;

#[cfg(feature = "support_model_renderer")] use crate::CENTRAL_COMMAND;
#[cfg(feature = "support_model_renderer")] use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
#[cfg(feature = "support_model_renderer")] use crate::GAME_SELECTED;
#[cfg(feature = "support_model_renderer")] use crate::packedfile_views::DataSource;
use crate::UI_STATE;
use crate::settings_helpers::{settings_bool, settings_set_raw_data};
use crate::utils::{qtr, tr};
use crate::views::table::{ITEM_HAS_VANILLA_VALUE, ITEM_ICON_CACHE, ITEM_ICON_PATH, ITEM_VANILLA_VALUE, ITEM_SOURCE_VALUE};

//---------------------------------------------------------------------------//
// Custom delegates stuff.
//---------------------------------------------------------------------------//

//extern "C" { fn new_search_match_item_delegate(table_view: *mut QObject, column: i32); }
//pub fn new_search_match_item_delegate_safe(table_view: &Ptr<QObject>, column: i32) {
//    unsafe { new_search_match_item_delegate(table_view.as_mut_raw_ptr(), column) }
//}

extern "C" { fn new_unit_variant_item_delegate(table_view: *mut QObject, column: i32); }
pub fn new_unit_variant_item_delegate_safe(table_view: &Ptr<QObject>, column: i32) {
    unsafe { new_unit_variant_item_delegate(table_view.as_mut_raw_ptr(), column) }
}

// This function replaces the default editor widget for reference columns with a combobox, so you can select the reference data.
extern "C" { fn new_combobox_item_delegate(table_view: *mut QObject, column: i32, list: *const QStringList, lookup_list: *const QStringList, is_editable: bool, timer: *mut QTimer, is_dark_theme_enabled: bool, has_filter: bool, is_right_side_mark_enabled: bool, enable_diff_markers: bool); }
pub fn new_combobox_item_delegate_safe(table_view: &Ptr<QObject>, column: i32, list: Ptr<QStringList>, lookup_list: Ptr<QStringList>, is_editable: bool, timer: &Ptr<QTimer>, has_filter: bool) {
    let is_dark_theme_enabled = settings_bool("use_dark_theme");
    let is_right_side_mark_enabled = settings_bool("use_right_size_markers");
    let enable_diff_markers = settings_bool("enable_diff_markers");
    unsafe { new_combobox_item_delegate(table_view.as_mut_raw_ptr(), column, list.as_raw_ptr(), lookup_list.as_raw_ptr(), is_editable, timer.as_mut_raw_ptr(), is_dark_theme_enabled, has_filter, is_right_side_mark_enabled, enable_diff_markers) }
}

// This function changes the default editor widget for I32 cells on tables with a numeric one.
extern "C" { fn new_spinbox_item_delegate(table_view: *mut QObject, column: i32, integer_type: i32, timer: *mut QTimer, is_dark_theme_enabled: bool, has_filter: bool, is_right_side_mark_enabled: bool, enable_diff_markers: bool); }
pub fn new_spinbox_item_delegate_safe(table_view: &Ptr<QObject>, column: i32, integer_type: i32, timer: &Ptr<QTimer>, has_filter: bool) {
    let is_dark_theme_enabled = settings_bool("use_dark_theme");
    let is_right_side_mark_enabled = settings_bool("use_right_size_markers");
    let enable_diff_markers = settings_bool("enable_diff_markers");
    unsafe { new_spinbox_item_delegate(table_view.as_mut_raw_ptr(), column, integer_type, timer.as_mut_raw_ptr(), is_dark_theme_enabled, has_filter, is_right_side_mark_enabled, enable_diff_markers) }
}

// This function changes the default editor widget for F32 cells on tables with a numeric one.
extern "C" { fn new_doublespinbox_item_delegate(table_view: *mut QObject, column: i32, timer: *mut QTimer, is_dark_theme_enabled: bool, has_filter: bool, is_right_side_mark_enabled: bool, enable_diff_markers: bool); }
pub fn new_doublespinbox_item_delegate_safe(table_view: &Ptr<QObject>, column: i32, timer: &Ptr<QTimer>, has_filter: bool) {
    let is_dark_theme_enabled = settings_bool("use_dark_theme");
    let is_right_side_mark_enabled = settings_bool("use_right_size_markers");
    let enable_diff_markers = settings_bool("enable_diff_markers");
    unsafe { new_doublespinbox_item_delegate(table_view.as_mut_raw_ptr(), column, timer.as_mut_raw_ptr(), is_dark_theme_enabled, has_filter, is_right_side_mark_enabled, enable_diff_markers) }
}

// This function changes the default editor widget for ColourRGB cells, to ensure the provided data is valid for the schema.
extern "C" { fn new_colour_item_delegate(table_view: *mut QObject, column: i32, timer: *mut QTimer, is_dark_theme_enabled: bool, has_filter: bool, is_right_side_mark_enabled: bool, enable_diff_markers: bool); }
pub fn new_colour_item_delegate_safe(table_view: &Ptr<QObject>, column: i32, timer: &Ptr<QTimer>, has_filter: bool) {
    let is_dark_theme_enabled = settings_bool("use_dark_theme");
    let is_right_side_mark_enabled = settings_bool("use_right_size_markers");
    let enable_diff_markers = settings_bool("enable_diff_markers");
    unsafe { new_colour_item_delegate(table_view.as_mut_raw_ptr(), column, timer.as_mut_raw_ptr(), is_dark_theme_enabled, has_filter, is_right_side_mark_enabled, enable_diff_markers) }
}

// This function changes the default editor widget for String cells, to ensure the provided data is valid for the schema.
extern "C" { fn new_qstring_item_delegate(table_view: *mut QObject, column: i32, timer: *mut QTimer, is_dark_theme_enabled: bool, has_filter: bool, is_right_side_mark_enabled: bool, enable_diff_markers: bool); }
pub fn new_qstring_item_delegate_safe(table_view: &Ptr<QObject>, column: i32, timer: &Ptr<QTimer>, has_filter: bool) {
    let is_dark_theme_enabled = settings_bool("use_dark_theme");
    let is_right_side_mark_enabled = settings_bool("use_right_size_markers");
    let enable_diff_markers = settings_bool("enable_diff_markers");
    unsafe { new_qstring_item_delegate(table_view.as_mut_raw_ptr(), column, timer.as_mut_raw_ptr(), is_dark_theme_enabled, has_filter, is_right_side_mark_enabled, enable_diff_markers) }
}

// This function changes the default delegate for all cell types that doesn't have a specific delegate already.
extern "C" { fn new_generic_item_delegate(table_view: *mut QObject, column: i32, timer: *mut QTimer, is_dark_theme_enabled: bool, has_filter: bool, is_right_side_mark_enabled: bool, enable_diff_markers: bool); }
pub fn new_generic_item_delegate_safe(table_view: &Ptr<QObject>, column: i32, timer: &Ptr<QTimer>, has_filter: bool) {
    let is_dark_theme_enabled = settings_bool("use_dark_theme");
    let is_right_side_mark_enabled = settings_bool("use_right_size_markers");
    let enable_diff_markers = settings_bool("enable_diff_markers");
    unsafe { new_generic_item_delegate(table_view.as_mut_raw_ptr(), column, timer.as_mut_raw_ptr(), is_dark_theme_enabled, has_filter, is_right_side_mark_enabled, enable_diff_markers) }
}

// This function changes the default delegate for all items in a Tips ListView.
extern "C" { fn new_tips_item_delegate(tree_view: *mut QObject, is_dark_theme_enabled: bool, has_filter: bool); }
pub fn new_tips_item_delegate_safe(tree_view: &Ptr<QObject>, has_filter: bool) {
    let is_dark_theme_enabled = settings_bool("use_dark_theme");
    unsafe { new_tips_item_delegate(tree_view.as_mut_raw_ptr(), is_dark_theme_enabled, has_filter) }
}

// This function changes the default delegate for all items in a TreeView.
extern "C" { fn new_tree_item_delegate(tree_view: *mut QObject, is_dark_theme_enabled: bool, has_filter: bool); }
pub fn new_tree_item_delegate_safe(tree_view: &Ptr<QObject>, has_filter: bool) {
    let is_dark_theme_enabled = settings_bool("use_dark_theme");
    unsafe { new_tree_item_delegate(tree_view.as_mut_raw_ptr(), is_dark_theme_enabled, has_filter) }
}

// This function setup the special filter used for the PackFile Contents `TreeView`.
extern "C" { fn new_treeview_filter(parent: *mut QObject) -> *mut QSortFilterProxyModel; }
pub fn new_treeview_filter_safe(parent: QPtr<QObject>) ->  QBox<QSortFilterProxyModel> {
    unsafe { QBox::from_raw(new_treeview_filter(parent.as_mut_raw_ptr())) }
}

// This function triggers the special filter used for the PackFile Contents `TreeView`. It has to be triggered here to work properly.
extern "C" { fn trigger_treeview_filter(filter: *const QSortFilterProxyModel, pattern: *mut QRegExp); }
pub fn trigger_treeview_filter_safe(filter: &QSortFilterProxyModel, pattern: &Ptr<QRegExp>) {
    unsafe { trigger_treeview_filter(filter, pattern.as_mut_raw_ptr()); }
}

// This function setup the special filter used for the TableViews.
extern "C" { fn new_tableview_filter(parent: *mut QObject) -> *mut QSortFilterProxyModel; }
pub fn new_tableview_filter_safe(parent: QPtr<QObject>) ->  QBox<QSortFilterProxyModel> {
    unsafe { QBox::from_raw(new_tableview_filter(parent.as_mut_raw_ptr())) }
}

// This function triggers the special filter used for the TableViews It has to be triggered here to work properly.
extern "C" { fn trigger_tableview_filter(filter: *const QSortFilterProxyModel, columns: *const QListOfInt, patterns: *const QStringList, use_nott: *const QListOfInt, regex: *const QListOfInt, case_sensitive: *const QListOfInt, show_blank_cells: *const QListOfInt, match_groups: *const QListOfInt, variant_to_search: *const QListOfInt, show_edited_cells: *const QListOfInt); }
pub unsafe fn trigger_tableview_filter_safe(filter: &QSortFilterProxyModel, columns: &[i32], patterns: Vec<Ptr<QString>>, use_nott: &[bool], regex: &[bool], case_sensitive: &[CaseSensitivity], show_blank_cells: &[bool], match_groups: &[i32], variant_to_search: &[i32], show_edited_cells: &[bool]) {
    let columns_qlist = QListOfInt::new();
    columns.iter().for_each(|x| columns_qlist.append_int(x));

    let patterns_qlist = QStringList::new();
    patterns.iter().for_each(|x| patterns_qlist.append_q_string(x.as_ref().unwrap()));

    let use_nott_qlist = QListOfInt::new();
    use_nott.iter().for_each(|x| use_nott_qlist.append_int(if *x { &1i32 } else { &0i32 }));

    let regex_qlist = QListOfInt::new();
    regex.iter().for_each(|x| regex_qlist.append_int(if *x { &1i32 } else { &0i32 }));

    let case_sensitive_qlist = QListOfInt::new();
    case_sensitive.iter().for_each(|x| case_sensitive_qlist.append_int(&x.to_int()));

    let show_blank_cells_qlist = QListOfInt::new();
    show_blank_cells.iter().for_each(|x| show_blank_cells_qlist.append_int(if *x { &1i32 } else { &0i32 }));

    let match_groups_qlist = QListOfInt::new();
    match_groups.iter().for_each(|x| match_groups_qlist.append_int(x));

    let variant_to_search_qlist = QListOfInt::new();
    variant_to_search.iter().for_each(|x| variant_to_search_qlist.append_int(x));

    let show_edited_cells_qlist = QListOfInt::new();
    show_edited_cells.iter().for_each(|x| show_edited_cells_qlist.append_int(if *x { &1i32 } else { &0i32 }));

    trigger_tableview_filter(filter, columns_qlist.into_ptr().as_raw_ptr(), patterns_qlist.into_ptr().as_raw_ptr(), use_nott_qlist.into_ptr().as_raw_ptr(), regex_qlist.into_ptr().as_raw_ptr(), case_sensitive_qlist.into_ptr().as_raw_ptr(), show_blank_cells_qlist.into_ptr().as_raw_ptr(), match_groups_qlist.into_ptr().as_raw_ptr(), variant_to_search_qlist.into_ptr().as_raw_ptr(), show_edited_cells_qlist.into_ptr().as_raw_ptr());
}

// This function allow us to create a QTreeView compatible with draggable items
extern "C" { fn new_packed_file_treeview(parent: *mut QWidget) -> *mut QTreeView; }
pub fn new_packed_file_treeview_safe(parent: QPtr<QWidget>) -> QPtr<QTreeView> {
    unsafe { QPtr::from_raw(new_packed_file_treeview(parent.as_mut_raw_ptr())) }
}

pub fn draggable_file_tree_view_drop_signal(widget: QPtr<QWidget>) -> Signal<(*const QModelIndex, i32)> {
    unsafe {
        Signal::new(
            ::cpp_core::Ref::from_raw(widget.as_raw_ptr()).expect("attempted to construct a null Ref"),
            c"2itemDrop(QModelIndex const &,int)",
        )
    }
}

// This function allow us to create a model compatible with draggable items
extern "C" { fn new_packed_file_model() -> *mut QStandardItemModel; }
pub fn new_packed_file_model_safe() -> QBox<QStandardItemModel> {
    unsafe { QBox::from_raw(new_packed_file_model()) }
}

// This function allow us to create a custom window.
extern "C" { fn new_q_main_window_custom(are_you_sure: extern "C" fn(*mut QMainWindow, bool) -> bool, is_dark_theme_enabled: bool) -> *mut QMainWindow; }
pub fn new_q_main_window_custom_safe(are_you_sure: extern "C" fn(*mut QMainWindow, bool) -> bool) -> QBox<QMainWindow> {
    let is_dark_theme_enabled = settings_bool("use_dark_theme");
    unsafe { QBox::from_raw(new_q_main_window_custom(are_you_sure, is_dark_theme_enabled)) }
}

pub fn main_window_drop_pack_signal(widget: QPtr<QWidget>) -> Signal<(*const ::qt_core::QStringList,)> {
    unsafe {
        Signal::new(
            ::cpp_core::Ref::from_raw(widget.as_raw_ptr()).expect("attempted to construct a null Ref"),
            c"2openPack(QStringList const &)",
        )
    }
}

// This function allow us to create a custom dialog.
#[cfg(feature = "enable_tools")] extern "C" { fn new_q_dialog_custom(parent: *mut QWidget, are_you_sure_dialog: extern "C" fn(*mut QDialog) -> bool) -> *mut QDialog; }
#[cfg(feature = "enable_tools")] pub fn new_q_dialog_custom_safe(parent: Ptr<QWidget>, are_you_sure_dialog: extern "C" fn(*mut QDialog) -> bool) -> QBox<QDialog> {
    unsafe { QBox::from_raw(new_q_dialog_custom(parent.as_mut_raw_ptr(), are_you_sure_dialog)) }
}

//---------------------------------------------------------------------------//
// i64 Spinbox stuff.
//---------------------------------------------------------------------------//

extern "C" { fn new_q_spinbox_i64(parent: *mut QWidget) -> *mut QAbstractSpinBox; }
pub fn new_q_spinbox_i64_safe(parent: &QPtr<QWidget>) -> QPtr<QAbstractSpinBox> {
    unsafe { QPtr::from_raw(new_q_spinbox_i64(parent.as_mut_raw_ptr())) }
}

extern "C" { fn value_q_spinbox_i64(widget: *mut QAbstractSpinBox) -> i64; }
pub fn value_q_spinbox_i64_safe(widget: &QPtr<QAbstractSpinBox>) -> i64 {
    unsafe { value_q_spinbox_i64(widget.as_mut_raw_ptr()) }
}

extern "C" { fn set_value_q_spinbox_i64(widget: *mut QAbstractSpinBox, value: i64); }
pub fn set_value_q_spinbox_i64_safe(widget: &QPtr<QAbstractSpinBox>, value: i64) {
    unsafe { set_value_q_spinbox_i64(widget.as_mut_raw_ptr(), value) }
}

extern "C" { fn set_min_q_spinbox_i64(widget: *mut QAbstractSpinBox, value: i64); }
pub fn set_min_q_spinbox_i64_safe(widget: &QPtr<QAbstractSpinBox>, value: i64) {
    unsafe { set_min_q_spinbox_i64(widget.as_mut_raw_ptr(), value) }
}

extern "C" { fn set_max_q_spinbox_i64(widget: *mut QAbstractSpinBox, value: i64); }
pub fn set_max_q_spinbox_i64_safe(widget: &QPtr<QAbstractSpinBox>, value: i64) {
    unsafe { set_max_q_spinbox_i64(widget.as_mut_raw_ptr(), value) }
}

//pub fn value_changed_q_spinbox_i64(widget: &QPtr<QAbstractSpinBox>) -> Signal<(i64,)> {
//    unsafe {
//        Signal::new(
//            ::cpp_core::Ref::from_raw(widget.as_raw_ptr()).expect("attempted to construct a null Ref"),
//            ::std::ffi::CStr::from_bytes_with_nul_unchecked(
//                b"2valueChanged(qlonglong const &)\0",
//            ),
//        )
//    }
//}

//---------------------------------------------------------------------------//
// Spoiler stuff.
//---------------------------------------------------------------------------//

extern "C" { fn new_spoiler(title: *const QString, animation_duration: i32, parent: *mut QWidget) -> *mut QWidget; }
pub fn new_spoiler_safe(title: &Ptr<QString>, animation_duration: i32, parent: &Ptr<QWidget>) -> QBox<QWidget> {
    unsafe { QBox::from_raw(new_spoiler(title.as_raw_ptr(), animation_duration, parent.as_mut_raw_ptr())) }
}

extern "C" { fn set_spoiler_layout(spoiler: *mut QWidget, layout: *const QLayout); }
pub fn set_spoiler_layout_safe(spoiler: &Ptr<QWidget>, layout: &Ptr<QLayout>) {
    unsafe { set_spoiler_layout(spoiler.as_mut_raw_ptr(), layout.as_mut_raw_ptr()) }
}

extern "C" { fn toggle_animated(spoiler: *mut QWidget); }
pub fn toggle_animated_safe(spoiler: &Ptr<QWidget>) {
    unsafe { toggle_animated(spoiler.as_mut_raw_ptr()) }
}

//---------------------------------------------------------------------------//
// Freezing Columns stuff.
//---------------------------------------------------------------------------//

// This function allows you to create a table capable of freezing columns.
extern "C" { fn new_tableview_frozen(parent: *mut QWidget, generate_tooltip_message: extern "C" fn(*mut QTableView, i32, i32) -> ()) -> *mut QTableView; }
pub fn new_tableview_frozen_safe(parent: &Ptr<QWidget>, generate_tooltip_message: extern "C" fn(*mut QTableView, i32, i32) -> ()) -> QBox<QTableView> {
    unsafe { QBox::from_raw(new_tableview_frozen(parent.as_mut_raw_ptr(), generate_tooltip_message)) }
}

// This function allows you to freeze/unfreeze a column.
extern "C" { fn toggle_freezer(table: *mut QTableView, column: i32); }
pub fn toggle_freezer_safe(table: &QBox<QTableView>, column: i32) {
    unsafe { toggle_freezer(table.as_mut_raw_ptr(), column) };
}

//---------------------------------------------------------------------------//
// KTextEditor stuff.
//---------------------------------------------------------------------------//

// This function allow us to create a complete KTextEditor.
extern "C" { fn new_text_editor(parent: *mut QWidget) -> *mut QWidget; }
pub fn new_text_editor_safe(parent: &QPtr<QWidget>) -> QBox<QWidget> {
    unsafe { QBox::from_raw(new_text_editor(parent.as_mut_raw_ptr())) }
}

// This function allow us to get the text from the provided KTextEditor.
extern "C" { fn get_text(document: *mut QWidget) -> *mut QString; }
pub fn get_text_safe(document: &QBox<QWidget>) -> Ptr<QString> {
    unsafe { Ptr::from_raw(get_text(document.as_mut_raw_ptr())) }
}

// This function allow us to set the text of the provided KTextEditor.
extern "C" { fn set_text(document: *mut QWidget, string: *mut QString, highlighting_mode: *mut QString); }
pub fn set_text_safe(document: &QPtr<QWidget>, string: &Ptr<QString>, highlighting_mode: &Ptr<QString>) {
    unsafe { set_text(document.as_mut_raw_ptr(), string.as_mut_raw_ptr(), highlighting_mode.as_mut_raw_ptr()) }
}

// This function triggers the config dialog for the KTextEditor.
extern "C" { fn open_text_editor_config(parent: *mut QWidget); }
pub fn open_text_editor_config_safe(parent: &Ptr<QWidget>) {
    unsafe { open_text_editor_config(parent.as_mut_raw_ptr()) }
}

// This function triggers the config dialog for the KTextEditor.
extern "C" { fn get_text_changed_dummy_widget(parent: *mut QWidget) -> *mut QLineEdit; }
pub fn get_text_changed_dummy_widget_safe(parent: &Ptr<QWidget>) -> Ptr<QLineEdit> {
    unsafe { Ptr::from_raw(get_text_changed_dummy_widget(parent.as_mut_raw_ptr())) }
}

// This function allows to scroll to an specific row in a KTextEditor.
extern "C" { fn scroll_to_row(parent: *mut QWidget, row_number: u64); }
pub fn scroll_to_row_safe(parent: &Ptr<QWidget>, row_number: u64) {
    unsafe { scroll_to_row(parent.as_mut_raw_ptr(), row_number) }
}

// This function returns the current row of the cursor in a KTextEditor.
extern "C" { fn cursor_row(parent: *mut QWidget) -> u64; }
pub fn cursor_row_safe(parent: &Ptr<QWidget>) -> u64 {
    unsafe { cursor_row(parent.as_mut_raw_ptr()) }
}

// This function allows to scroll to an specific position in a KTextEditor and select a range.
extern "C" { fn scroll_to_pos_and_select(parent: *mut QWidget, start_row: u64, start_column: u64, end_row: u64, end_column: u64); }
pub fn scroll_to_pos_and_select_safe(parent: &Ptr<QWidget>, start_row: u64, start_column: u64, end_row: u64, end_column: u64) {
    unsafe { scroll_to_pos_and_select(parent.as_mut_raw_ptr(), start_row, start_column, end_row, end_column) }
}

//---------------------------------------------------------------------------//
// KColorCombo stuff.
//---------------------------------------------------------------------------//

// This function allow us to get the QColor from the provided KColorCombo.
#[cfg(feature = "enable_tools")] extern "C" { fn get_color(view: *mut QWidget) -> u32; }
#[cfg(feature = "enable_tools")] pub fn get_color_safe(view: &Ptr<QWidget>) -> CppBox<QColor> {
    unsafe { QColor::from_rgba(get_color(view.as_mut_raw_ptr())) }
}

// This function allow us to set the QColor of the provided KColorCombo.
#[cfg(feature = "enable_tools")] extern "C" { fn set_color(view: *mut QWidget, color: *mut QColor); }
#[cfg(feature = "enable_tools")] pub fn set_color_safe(view: &Ptr<QWidget>, color: &Ptr<QColor>) {
    unsafe { set_color(view.as_mut_raw_ptr(), color.as_mut_raw_ptr()) }
}

//---------------------------------------------------------------------------//
// KLineEdit stuff.
//---------------------------------------------------------------------------//

// This function allow us to pre-configure a KLineEdit.
extern "C" { fn kline_edit_configure(view: *mut QWidget); }
pub fn kline_edit_configure_safe(view: &Ptr<QWidget>) {
    unsafe { kline_edit_configure(view.as_mut_raw_ptr()) };
}

//---------------------------------------------------------------------------//
// KMessageWidget stuff.
//---------------------------------------------------------------------------//

// This function allow us to create a KMessageWidget.
extern "C" { fn kmessage_widget_new(widget: *mut QWidget) -> *mut QWidget; }
pub fn kmessage_widget_new_safe(widget: &Ptr<QWidget>) -> QPtr<QWidget> {
    unsafe { QPtr::new(kmessage_widget_new(widget.as_mut_raw_ptr())) }
}

// This function allow us to close a KMessageWidget.
extern "C" { fn kmessage_widget_close(widget: *mut QWidget); }
pub fn kmessage_widget_close_safe(widget: &Ptr<QWidget>) {
    unsafe { kmessage_widget_close(widget.as_mut_raw_ptr()) }
}

// This function allow us to check if a KMessageWidget is closed.
extern "C" { fn kmessage_widget_is_closed(widget: *mut QWidget) -> bool; }
pub fn kmessage_widget_is_closed_safe(widget: &Ptr<QWidget>) -> bool {
    unsafe { kmessage_widget_is_closed(widget.as_mut_raw_ptr()) }
}

// This function allow us to set the text of the provided KMessageWidget, and se its type to Error.
#[allow(dead_code)]
extern "C" { fn kmessage_widget_set_error(widget: *mut QWidget, text: *const QString); }
#[allow(dead_code)]
pub fn kmessage_widget_set_error_safe(widget: &Ptr<QWidget>, text: Ptr<QString>) {
    unsafe { kmessage_widget_set_error(widget.as_mut_raw_ptr(), text.as_raw_ptr()) }
}

// This function allow us to set the text of the provided KMessageWidget, and se its type to Warning.
#[allow(dead_code)]
extern "C" { fn kmessage_widget_set_warning(widget: *mut QWidget, text: *const QString); }
#[allow(dead_code)]
pub fn kmessage_widget_set_warning_safe(widget: &Ptr<QWidget>, text: Ptr<QString>) {
    unsafe { kmessage_widget_set_warning(widget.as_mut_raw_ptr(), text.as_raw_ptr()) }
}

// This function allow us to set the text of the provided KMessageWidget, and se its type to Info.
#[allow(dead_code)]
extern "C" { fn kmessage_widget_set_info(widget: *mut QWidget, text: *const QString); }
#[allow(dead_code)]
pub fn kmessage_widget_set_info_safe(widget: &Ptr<QWidget>, text: Ptr<QString>) {
    unsafe { kmessage_widget_set_info(widget.as_mut_raw_ptr(), text.as_raw_ptr()) }
}

//---------------------------------------------------------------------------//
// KShortcutsDialog stuff.
//---------------------------------------------------------------------------//

extern "C" { fn shortcut_collection_init(widget: *mut QWidget, shortcuts: *mut QListOfQObject); }
pub fn shortcut_collection_init_safe(widget: &Ptr<QWidget>, shortcuts: Ptr<QListOfQObject>) {
    unsafe { shortcut_collection_init(widget.as_mut_raw_ptr(), shortcuts.as_mut_raw_ptr()) }
}

extern "C" { fn shortcut_action(shortcuts: *const QListOfQObject, action_group: *const QString, action_name: *const QString) -> *const QAction; }
pub fn shortcut_action_safe(shortcuts: Ptr<QListOfQObject>, action_group: Ptr<QString>, action_name: Ptr<QString>) -> QPtr<QAction> {
    unsafe { QPtr::from_raw(shortcut_action(shortcuts.as_raw_ptr(), action_group.as_raw_ptr(), action_name.as_raw_ptr())) }
}

extern "C" { fn kshortcut_dialog_init(widget: *mut QWidget, shortcuts: *mut QListOfQObject); }
pub fn kshortcut_dialog_init_safe(widget: &Ptr<QWidget>, shortcuts: Ptr<QListOfQObject>) {
    unsafe { kshortcut_dialog_init(widget.as_mut_raw_ptr(), shortcuts.as_mut_raw_ptr()) }
}

//---------------------------------------------------------------------------//
// Image stuff.
//---------------------------------------------------------------------------//

// This function allow us to create a QLabel whose QPixmap gets resized with the resize events of the label.
extern "C" { fn new_resizable_label(parent: *mut QWidget, pixmap: *mut QPixmap) -> *mut QLabel; }
pub fn new_resizable_label_safe(parent: &Ptr<QWidget>, pixmap: &Ptr<QPixmap>) -> QPtr<QLabel> {
    unsafe { QPtr::from_raw(new_resizable_label(parent.as_mut_raw_ptr(), pixmap.as_mut_raw_ptr())) }
}

extern "C" { fn set_pixmap_on_resizable_label(label: *mut QLabel, pixmap: *mut QPixmap); }
pub fn set_pixmap_on_resizable_label_safe(label: &Ptr<QLabel>, pixmap: &Ptr<QPixmap>) {
    unsafe { set_pixmap_on_resizable_label(label.as_mut_raw_ptr(), pixmap.as_mut_raw_ptr()); }
}

//---------------------------------------------------------------------------//
// Rigidmodel stuff.
//---------------------------------------------------------------------------//

#[cfg(feature = "support_model_renderer")]
extern "C" { fn CreateQRenderingWidget(parent: *mut QWidget, gameIdString: *mut QString, AssetFetchCallBack: extern fn (*mut QListOfQString, *mut QListOfQByteArray), AnimPathsBySkeletonCallBack: extern fn (*mut QString, *mut QListOfQString)) -> *mut QWidget; }
#[cfg(feature = "support_model_renderer")]
pub fn create_q_rendering_widget(parent: &Ptr<QWidget>) -> Result<QBox<QWidget>> {
    let game = QString::from_std_str(GAME_SELECTED.read().unwrap().key());
    let widget = unsafe { CreateQRenderingWidget(parent.as_mut_raw_ptr(), game.as_mut_raw_ptr(), assets_request_callback, anim_paths_by_skeleton_callback) };
    if widget.is_null() {
        Err(anyhow!("Error creating rendering widget. Check log for more info/reporting it as a bug."))
    } else {
        unsafe { Ok(QBox::from_raw(widget)) }
    }
}

#[cfg(feature = "support_model_renderer")]
extern "C" { fn AddNewPrimaryAsset(pQRenderWiget: *mut QWidget, assetsPath: *mut QString, assetData: *mut QByteArray, outErrorString: *mut QString); }
#[cfg(feature = "support_model_renderer")]
pub unsafe fn add_new_primary_asset(widget: &Ptr<QWidget>, asset_path: &str, asset_data: &[u8]) -> Result<()> {
    let asset_path = QString::from_std_str(asset_path);
    let asset_data = QByteArray::from_slice(asset_data);
    let error = QString::new();
    AddNewPrimaryAsset(widget.as_mut_raw_ptr(), asset_path.as_mut_raw_ptr(), asset_data.as_mut_raw_ptr(), error.as_mut_raw_ptr());

    if error.is_empty() {
        Ok(())
    } else {
        Err(anyhow!(error.to_std_string()))
    }
}

#[cfg(feature = "support_model_renderer")]
extern "C" { fn SetAssetFolder(folder: *mut QString); }
#[cfg(feature = "support_model_renderer")]
pub unsafe fn set_asset_folder(folder: &str) {
    let folder = QString::from_std_str(folder);
    SetAssetFolder(folder.as_mut_raw_ptr())
}

#[cfg(feature = "support_model_renderer")]
extern "C" { fn SetLogFolder(folder: *mut QString); }
#[cfg(feature = "support_model_renderer")]
pub unsafe fn set_log_folder(folder: &str) {
    let folder = QString::from_std_str(folder);
    SetLogFolder(folder.as_mut_raw_ptr())
}

#[cfg(feature = "support_model_renderer")]
extern "C" { fn PauseRendering(pQRendeeWiget: *mut QWidget); }
#[cfg(feature = "support_model_renderer")]
pub unsafe fn pause_rendering(widget: &Ptr<QWidget>) {
    PauseRendering(widget.as_mut_raw_ptr())
}

#[cfg(feature = "support_model_renderer")]
extern "C" { fn ResumeRendering(pQRendeeWiget: *mut QWidget); }
#[cfg(feature = "support_model_renderer")]
pub unsafe fn resume_rendering(widget: &Ptr<QWidget>) {
    ResumeRendering(widget.as_mut_raw_ptr())
}

#[cfg(feature = "support_model_renderer")]
pub extern fn assets_request_callback(missing_files: *mut QListOfQString, out: *mut QListOfQByteArray) {
    unsafe {
        let missing_files = missing_files.as_ref().unwrap();
        let out = out.as_mut().unwrap();

        let mut paths = vec![];
        for i in 0..missing_files.count_0a() {
            let mut path = missing_files.at(i).to_std_string().replace("\\", "/");

            // Fix for receiving paths with padding.
            if let Some(index) = path.find("\0") {
                let _ = path.split_off(index);
            }

            if path.starts_with("/") {
                path.remove(0);
            }

            paths.push(ContainerPath::File(path));
        }

        log::info!("Paths requested by model renderer: {:#?}", &paths.iter().map(|x| format!(" - {}", x.path_raw())).collect::<Vec<_>>());

        let receiver = CENTRAL_COMMAND.read().unwrap().send_background(Command::GetRFilesFromAllSources(paths.clone(), true));
        let response = CentralCommand::recv(&receiver);
        match response {
            Response::HashMapDataSourceHashMapStringRFile(mut files) => {
                let mut files_merge = HashMap::new();
                if let Some(files) = files.remove(&DataSource::GameFiles) {
                    files_merge.extend(files);
                }

                if let Some(files) = files.remove(&DataSource::ParentFiles) {
                    files_merge.extend(files);
                }

                if let Some(files) = files.remove(&DataSource::PackFile) {
                    files_merge.extend(files);
                }

                // Files have to go in the same order they came.
                // Missing or empty files just have to have an empty byte array.
                for path in &paths {
                    match files_merge.get_mut(&path.path_raw().to_lowercase()) {
                        Some(file) => {
                            match file.load() {
                                Ok(_) => match file.cached() {
                                    Ok(data) => {
                                        let data = QByteArray::from_slice(data);
                                        out.append_q_byte_array(&data);
                                    }
                                    Err(_) => out.append_q_byte_array(&QByteArray::new()),
                                }
                                Err(_) => out.append_q_byte_array(&QByteArray::new()),
                            }
                        }
                        None => out.append_q_byte_array(&QByteArray::new()),
                    }
                }
            },
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        };
    }
}

#[cfg(feature = "support_model_renderer")]
pub extern fn anim_paths_by_skeleton_callback(skeleton_name: *mut QString, out: *mut QListOfQString) {
    unsafe {
        let skeleton_name = skeleton_name.as_ref().unwrap().to_std_string();
        let out = out.as_mut().unwrap();

        log::info!("Anim Paths requested for skeleton: {}", &skeleton_name);

        let receiver = CENTRAL_COMMAND.read().unwrap().send_background(Command::GetAnimPathsBySkeletonName(skeleton_name));
        let response = CentralCommand::recv(&receiver);
        match response {
            Response::HashSetString(paths) => {
                for path in &paths {
                    out.append_q_string(&QString::from_std_str(path));
                }
            },
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        };
    }
}

//---------------------------------------------------------------------------//
// Special functions.
//---------------------------------------------------------------------------//

/// This function allow us to create a dialog when trying to close the main window.
pub extern "C" fn are_you_sure(main_window: *mut QMainWindow, is_delete_my_mod: bool) -> bool {
    unsafe {
        if !is_delete_my_mod {
            settings_set_raw_data("geometry", &main_window.as_ref().unwrap().save_geometry().as_slice().iter().map(|x| *x as u8).collect::<Vec<_>>());
            settings_set_raw_data("windowState", &main_window.as_ref().unwrap().save_state_0a().as_slice().iter().map(|x| *x as u8).collect::<Vec<_>>());
        }
    }

    let title = qtr("rpfm_title");
    let message = if is_delete_my_mod {
        qtr("delete_mymod_0")
    } else if UI_STATE.get_is_modified() {
        qtr("delete_mymod_1")
    }

    // In any other situation... just return true and forget about the dialog.
    else { return true };

    // If we're closing the main window, save the geometry to the settings before closing it.
    unsafe {

        // Create the dialog and run it (Yes => 3, No => 4).
        QMessageBox::from_2_q_string_icon3_int_q_widget(
            &title,
            &message,
            q_message_box::Icon::Warning,
            65536, // No
            16384, // Yes
            1, // By default, select yes.
            main_window,
        ).exec() == 3
    }
}

/// This function allow us to create a dialog when trying to close another dialog.
#[cfg(feature = "enable_tools")] pub extern "C" fn are_you_sure_dialog(dialog: *mut QDialog) -> bool {
    let title = qtr("rpfm_title");
    let message = qtr("close_tool");

    // Create the dialog and run it (Yes => 3, No => 4).
    unsafe { QMessageBox::from_2_q_string_icon3_int_q_widget(
        &title,
        &message,
        q_message_box::Icon::Warning,
        65536, // No
        16384, // Yes
        1, // By default, select yes.
        dialog,
    ).exec() == 3 }
}

pub extern "C" fn generate_tooltip_message(view: *mut QTableView, global_pos_x: i32, global_pos_y: i32) {
    unsafe {
        let view = view.as_ref().unwrap();
        let global_pos = QPoint::new_2a(global_pos_x, global_pos_y);
        if view.under_mouse() {

            let filter_index = view.index_at(&view.viewport().map_from_global(&global_pos));
            if filter_index.is_valid() {

                let filter = view.model().static_downcast::<QSortFilterProxyModel>();
                let model = filter.source_model().static_downcast::<QStandardItemModel>();

                let model_index = filter.map_to_source(&filter_index);
                if model_index.is_valid() {

                    let item = model.item_from_index(&model_index);
                    model.block_signals(true);

                    // Only generate the icon base64 if we don't have one generated and the item has an icon.
                    //
                    // Further updates of this data need to be done through a dataChanged signal.
                    if model_index.data_1a(ITEM_ICON_CACHE).is_null() && !item.icon().is_null() {
                        let icon = item.icon();
                        let image = icon.pixmap_q_size(icon.available_sizes_0a().at(0)).to_image();
                        let bytes = QByteArray::new();
                        let buffer = QBuffer::from_q_byte_array(&bytes);

                        image.save_q_io_device_char(&buffer, QString::from_std_str("PNG").to_latin1().data());
                        item.set_data_2a(&QVariant::from_q_string(&QString::from_q_byte_array(&bytes.to_base64_0a())), ITEM_ICON_CACHE);
                    }

                    // Store the original tooltip elsewere so we can re-access it.
                    let mut tooltip_string = String::new();
                    let source_value = item.data_1a(ITEM_SOURCE_VALUE);
                    let has_vanilla_value = item.data_1a(ITEM_HAS_VANILLA_VALUE);
                    let vanilla_value = item.data_1a(ITEM_VANILLA_VALUE);

                    // Put toghether the message.
                    if !has_vanilla_value.is_null() &&
                        !vanilla_value.is_null() &&
                        has_vanilla_value.to_bool() {
                        tooltip_string.push_str(&tr("vanilla_data").replacen("{}", &vanilla_value.to_string().to_std_string(), 1));
                    }

                    if !source_value.is_null() {
                        if !tooltip_string.is_empty() {
                            tooltip_string.push_str("<br/>");
                        }

                        tooltip_string.push_str(&tr("original_data").replacen("{}", &source_value.to_string().to_std_string(), 1));

                        let icon_data = model_index.data_1a(ITEM_ICON_CACHE);
                        let image_path = item.data_1a(ITEM_ICON_PATH);
                        if !image_path.is_null() && !icon_data.is_null() {
                            tooltip_string.push_str(&format!("<br/>Image path: {}<br/><img src=\"data:image/png;base64, {}\"/>", image_path.to_string().to_std_string(), icon_data.to_string().to_std_string()));

                        }
                    }

                    if !tooltip_string.is_empty() {
                        item.set_tool_tip(&QString::from_std_str(tooltip_string));
                    }

                    model.block_signals(false);
                }
            }
        }
    }
}
