//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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

use qt_widgets::QLabel;
use qt_widgets::QLayout;
use qt_widgets::QLineEdit;
use qt_widgets::QMainWindow;
use qt_widgets::{QMessageBox, q_message_box};
use qt_widgets::QTableView;
use qt_widgets::QWidget;

use qt_gui::QColor;

#[cfg(feature = "support_modern_dds")]
use qt_gui::QImage;
use qt_gui::QPixmap;
use qt_gui::QStandardItemModel;

#[cfg(any(feature = "support_rigidmodel", feature = "support_modern_dds"))]
use qt_core::QByteArray;

use qt_core::QAbstractItemModel;
use qt_core::QBox;
use qt_core::QObject;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QStringList;
use qt_core::QPtr;
use qt_core::QTimer;
use qt_core::QListOfInt;
use qt_core::CaseSensitivity;

use cpp_core::CppBox;
use cpp_core::Ptr;

#[cfg(feature = "support_rigidmodel")]
use rpfm_error::{Result, ErrorKind};

use crate::locale::qtr;
use crate::settings_ui::backend::*;
use crate::UI_STATE;

//---------------------------------------------------------------------------//
// Custom delegates stuff.
//---------------------------------------------------------------------------//

// This function replaces the default editor widget for reference columns with a combobox, so you can select the reference data.
extern "C" { fn new_combobox_item_delegate(table_view: *mut QObject, column: i32, list: *const QStringList, is_editable: bool, timer: *mut QTimer, is_dark_theme_enabled: bool, has_filter: bool, is_right_side_mark_enabled: bool); }
pub fn new_combobox_item_delegate_safe(table_view: &Ptr<QObject>, column: i32, list: Ptr<QStringList>, is_editable: bool, timer: &Ptr<QTimer>, has_filter: bool) {
    let is_dark_theme_enabled = setting_bool("use_dark_theme");
    let is_right_side_mark_enabled = setting_bool("use_right_size_markers");
    unsafe { new_combobox_item_delegate(table_view.as_mut_raw_ptr(), column, list.as_raw_ptr(), is_editable, timer.as_mut_raw_ptr(), is_dark_theme_enabled, has_filter, is_right_side_mark_enabled) }
}

// This function changes the default editor widget for I32/64 cells on tables with a numeric one.
extern "C" { fn new_spinbox_item_delegate(table_view: *mut QObject, column: i32, integer_type: i32, timer: *mut QTimer, is_dark_theme_enabled: bool, has_filter: bool, is_right_side_mark_enabled: bool); }
pub fn new_spinbox_item_delegate_safe(table_view: &Ptr<QObject>, column: i32, integer_type: i32, timer: &Ptr<QTimer>, has_filter: bool) {
    let is_dark_theme_enabled = setting_bool("use_dark_theme");
    let is_right_side_mark_enabled = setting_bool("use_right_size_markers");
    unsafe { new_spinbox_item_delegate(table_view.as_mut_raw_ptr(), column, integer_type, timer.as_mut_raw_ptr(), is_dark_theme_enabled, has_filter, is_right_side_mark_enabled) }
}

// This function changes the default editor widget for F32 cells on tables with a numeric one.
extern "C" { fn new_doublespinbox_item_delegate(table_view: *mut QObject, column: i32, timer: *mut QTimer, is_dark_theme_enabled: bool, has_filter: bool, is_right_side_mark_enabled: bool); }
pub fn new_doublespinbox_item_delegate_safe(table_view: &Ptr<QObject>, column: i32, timer: &Ptr<QTimer>, has_filter: bool) {
    let is_dark_theme_enabled = setting_bool("use_dark_theme");
    let is_right_side_mark_enabled = setting_bool("use_right_size_markers");
    unsafe { new_doublespinbox_item_delegate(table_view.as_mut_raw_ptr(), column, timer.as_mut_raw_ptr(), is_dark_theme_enabled, has_filter, is_right_side_mark_enabled) }
}

// This function changes the default editor widget for ColourRGB cells, to ensure the provided data is valid for the schema.
extern "C" { fn new_colour_item_delegate(table_view: *mut QObject, column: i32, timer: *mut QTimer, is_dark_theme_enabled: bool, has_filter: bool, is_right_side_mark_enabled: bool); }
pub fn new_colour_item_delegate_safe(table_view: &Ptr<QObject>, column: i32, timer: &Ptr<QTimer>, has_filter: bool) {
    let is_dark_theme_enabled = setting_bool("use_dark_theme");
    let is_right_side_mark_enabled = setting_bool("use_right_size_markers");
    unsafe { new_colour_item_delegate(table_view.as_mut_raw_ptr(), column, timer.as_mut_raw_ptr(), is_dark_theme_enabled, has_filter, is_right_side_mark_enabled) }
}

// This function changes the default editor widget for String cells, to ensure the provided data is valid for the schema.
extern "C" { fn new_qstring_item_delegate(table_view: *mut QObject, column: i32, timer: *mut QTimer, is_dark_theme_enabled: bool, has_filter: bool, is_right_side_mark_enabled: bool); }
pub fn new_qstring_item_delegate_safe(table_view: &Ptr<QObject>, column: i32, timer: &Ptr<QTimer>, has_filter: bool) {
    let is_dark_theme_enabled = setting_bool("use_dark_theme");
    let is_right_side_mark_enabled = setting_bool("use_right_size_markers");
    unsafe { new_qstring_item_delegate(table_view.as_mut_raw_ptr(), column, timer.as_mut_raw_ptr(), is_dark_theme_enabled, has_filter, is_right_side_mark_enabled) }
}

// This function changes the default delegate for all cell types that doesn't have a specific delegate already.
extern "C" { fn new_generic_item_delegate(table_view: *mut QObject, column: i32, timer: *mut QTimer, is_dark_theme_enabled: bool, has_filter: bool, is_right_side_mark_enabled: bool); }
pub fn new_generic_item_delegate_safe(table_view: &Ptr<QObject>, column: i32, timer: &Ptr<QTimer>, has_filter: bool) {
    let is_dark_theme_enabled = setting_bool("use_dark_theme");
    let is_right_side_mark_enabled = setting_bool("use_right_size_markers");
    unsafe { new_generic_item_delegate(table_view.as_mut_raw_ptr(), column, timer.as_mut_raw_ptr(), is_dark_theme_enabled, has_filter, is_right_side_mark_enabled) }
}

// This function changes the default delegate for all items in a Tips ListView.
extern "C" { fn new_tips_item_delegate(tree_view: *mut QObject, is_dark_theme_enabled: bool, has_filter: bool); }
pub fn new_tips_item_delegate_safe(tree_view: &Ptr<QObject>, has_filter: bool) {
    let is_dark_theme_enabled = setting_bool("use_dark_theme");
    unsafe { new_tips_item_delegate(tree_view.as_mut_raw_ptr(), is_dark_theme_enabled, has_filter) }
}

// This function changes the default delegate for all items in a TreeView.
extern "C" { fn new_tree_item_delegate(tree_view: *mut QObject, is_dark_theme_enabled: bool, has_filter: bool); }
pub fn new_tree_item_delegate_safe(tree_view: &Ptr<QObject>, has_filter: bool) {
    let is_dark_theme_enabled = setting_bool("use_dark_theme");
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
extern "C" { fn trigger_tableview_filter(filter: *const QSortFilterProxyModel, columns: *const QListOfInt, patterns: *const QStringList, case_sensitive: *const QListOfInt, show_blank_cells: *const QListOfInt, match_groups: *const QListOfInt); }
pub unsafe fn trigger_tableview_filter_safe(filter: &QSortFilterProxyModel, columns: &[i32], patterns: Vec<Ptr<QString>>, case_sensitive: &[CaseSensitivity], show_blank_cells: &[bool], match_groups: &[i32]) {
    let columns_qlist = QListOfInt::new();
    columns.iter().for_each(|x| columns_qlist.append_int(x));

    let patterns_qlist = QStringList::new();
    patterns.iter().for_each(|x| patterns_qlist.append_q_string(x.as_ref().unwrap()));

    let case_sensitive_qlist = QListOfInt::new();
    case_sensitive.iter().for_each(|x| case_sensitive_qlist.append_int(&x.to_int()));

    let show_blank_cells_qlist = QListOfInt::new();
    show_blank_cells.iter().for_each(|x| show_blank_cells_qlist.append_int(if *x { &1i32 } else { &0i32 }));

    let match_groups_qlist = QListOfInt::new();
    match_groups.iter().for_each(|x| match_groups_qlist.append_int(x));

    trigger_tableview_filter(filter, columns_qlist.into_ptr().as_raw_ptr(), patterns_qlist.into_ptr().as_raw_ptr(), case_sensitive_qlist.into_ptr().as_raw_ptr(), show_blank_cells_qlist.into_ptr().as_raw_ptr(), match_groups_qlist.into_ptr().as_raw_ptr());
}


// This function allow us to create a model compatible with draggable items
extern "C" { fn new_packed_file_model() -> *mut QStandardItemModel; }
pub fn new_packed_file_model_safe() -> QBox<QStandardItemModel> {
    unsafe { QBox::from_raw(new_packed_file_model()) }
}

// This function allow us to create a custom window.
extern "C" { fn new_q_main_window_custom(are_you_sure: extern fn(*mut QMainWindow, bool) -> bool) -> *mut QMainWindow; }
pub fn new_q_main_window_custom_safe(are_you_sure: extern fn(*mut QMainWindow, bool) -> bool) -> QBox<QMainWindow> {
    unsafe { QBox::from_raw(new_q_main_window_custom(are_you_sure)) }
}

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
extern "C" { fn new_tableview_frozen(parent: *mut QWidget) -> *mut QTableView; }
extern "C" { fn get_frozen_view(table_view: *mut QTableView) -> *mut QTableView; }
pub fn new_tableview_frozen_safe(parent: &Ptr<QWidget>) -> (QBox<QTableView>, QBox<QTableView>) {
    let table_view_normal = unsafe { new_tableview_frozen(parent.as_mut_raw_ptr()) };
    let table_view_frozen = unsafe { get_frozen_view(table_view_normal) };
    unsafe { (QBox::from_raw(table_view_normal), QBox::from_raw(table_view_frozen)) }
}

// This function allows you to load data to a table capable of freezing columns.
extern "C" { fn set_data_model(table: *mut QTableView, model: *mut QAbstractItemModel); }
pub fn set_frozen_data_model_safe(table: &Ptr<QTableView>, model: &Ptr<QAbstractItemModel>) {
    unsafe { set_data_model(table.as_mut_raw_ptr(), model.as_mut_raw_ptr()) };
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
pub fn get_text_changed_dummy_widget_safe<'a>(parent: &Ptr<QWidget>) -> Ptr<QLineEdit> {
    unsafe { Ptr::from_raw(get_text_changed_dummy_widget(parent.as_mut_raw_ptr())) }
}

//---------------------------------------------------------------------------//
// KColorCombo stuff.
//---------------------------------------------------------------------------//

// This function allow us to get the QColor from the provided KColorCombo.
extern "C" { fn get_color(view: *mut QWidget) -> u32; }
pub fn get_color_safe(view: &Ptr<QWidget>) -> CppBox<QColor> {
    unsafe { QColor::from_rgba(get_color(view.as_mut_raw_ptr())) }
}

// This function allow us to set the QColor of the provided KColorCombo.
extern "C" { fn set_color(view: *mut QWidget, color: *mut QColor); }
pub fn set_color_safe(view: &Ptr<QWidget>, color: &Ptr<QColor>) {
    unsafe { set_color(view.as_mut_raw_ptr(), color.as_mut_raw_ptr()) }
}

//---------------------------------------------------------------------------//
// KMessageWidget stuff.
//---------------------------------------------------------------------------//

// This function allow us to close a KMessageWidget.
extern "C" { fn kmessage_widget_close(widget: *mut QWidget); }
pub fn kmessage_widget_close_safe(widget: &Ptr<QWidget>) {
    unsafe { kmessage_widget_close(widget.as_mut_raw_ptr()) }
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

// This function allow us to create a QImage with the contents of a DDS Texture.
#[cfg(feature = "support_modern_dds")]
extern "C" { fn getDDS_QImage(data: *const QByteArray) -> *mut QImage; }
#[cfg(feature = "support_modern_dds")]
pub fn get_dds_qimage(data: &Ptr<QByteArray>) -> Ptr<QImage> {
    unsafe { Ptr::from_raw(getDDS_QImage(data.as_mut_raw_ptr())) }
}

//---------------------------------------------------------------------------//
// Rigidmodel stuff.
//---------------------------------------------------------------------------//

// This function allow us to create a complete RigidModel view.
#[cfg(feature = "support_rigidmodel")]
extern "C" { fn createRMV2Widget(parent: *mut QWidget) -> *mut QWidget; }
#[cfg(feature = "support_rigidmodel")]
pub fn new_rigid_model_view_safe(parent: &Ptr<QWidget>) -> QBox<QWidget> {
    unsafe { QBox::from_raw(createRMV2Widget(parent.as_mut_raw_ptr())) }
}

// This function allow us to get the data from a Rigidmodel view.
#[cfg(feature = "support_rigidmodel")]
extern "C" { fn getRMV2Data(parent: *mut QWidget, data: *mut QByteArray) -> bool; }
#[cfg(feature = "support_rigidmodel")]
pub fn get_rigid_model_from_view_safe(parent: &QBox<QWidget>) -> Result<CppBox<QByteArray>> {
    unsafe {
        let data = QByteArray::new();
        if getRMV2Data(parent.as_mut_raw_ptr(), data.as_mut_raw_ptr()) {
            Ok(data)
        } else {
            let error = get_last_rigid_model_error(&parent.as_ptr())?;
            log::warn!("Error setting rigid data: {:?}:", error);
            Err(ErrorKind::GenericHTMLError(error).into())
        }
    }
}

// This function allow us to manually load data into a RigidModel View.
#[cfg(feature = "support_rigidmodel")]
extern "C" { fn setRMV2Data(parent: *mut QWidget, data: *const QByteArray) -> bool; }
#[cfg(feature = "support_rigidmodel")]
pub fn set_rigid_model_view_safe(parent: &Ptr<QWidget>, data: &Ptr<QByteArray>) -> Result<()> {
    unsafe {
        if setRMV2Data(parent.as_mut_raw_ptr(), data.as_raw_ptr()) {
            Ok(())
        } else {
            let error = get_last_rigid_model_error(parent)?;
            log::warn!("Error setting rigid data: {:?}:", error);
            Err(ErrorKind::GenericHTMLError(error).into())
        }
    }
}

// This function allow us to get the last error reported by the lib.
#[cfg(feature = "support_rigidmodel")]
extern "C" { fn getLastErrorString(parent: *mut QWidget, string: *mut QString) -> bool; }
#[cfg(feature = "support_rigidmodel")]
pub fn get_last_rigid_model_error(parent: &Ptr<QWidget>) -> Result<String> {
    unsafe {
        let string = QString::new();
        if getLastErrorString(parent.as_mut_raw_ptr(), string.as_mut_raw_ptr()) {
            Ok(string.to_std_string())
        } else {
            Err(ErrorKind::RigidModelParseError.into())
        }
    }
}

//---------------------------------------------------------------------------//
// Special functions.
//---------------------------------------------------------------------------//

/// This function allow us to create a dialog when trying to close the main window.
pub extern fn are_you_sure(main_window: *mut QMainWindow, is_delete_my_mod: bool) -> bool {
    let title = qtr("rpfm_title");
    let message = if is_delete_my_mod { qtr("delete_mymod_0") }
    else if UI_STATE.get_is_modified() { qtr("delete_mymod_1") }

    // In any other situation... just return true and forget about the dialog.
    else { return true };

    // Create the dialog and run it (Yes => 3, No => 4).
    unsafe { QMessageBox::from_2_q_string_icon3_int_q_widget(
        &title,
        &message,
        q_message_box::Icon::Warning,
        65536, // No
        16384, // Yes
        1, // By default, select yes.
        main_window,
    ).exec() == 3 }
}
