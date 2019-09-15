//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
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

use qt_widgets::table_view::TableView;

use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::object::Object;
use qt_core::reg_exp::RegExp;
use qt_core::sort_filter_proxy_model::SortFilterProxyModel;
use qt_core::string_list::StringList;

/// This function replaces the default editor widget for reference columns with a combobox, so you can select the reference data.
extern "C" { pub fn new_combobox_item_delegate(table_view: *mut Object, column: i32, list: *const StringList, is_editable: bool); }

/// This function changes the default editor widget for I32/64 cells on tables with a numeric one.
extern "C" { pub fn new_spinbox_item_delegate(table_view: *mut Object, column: i32, integer_type: i32); }

/// This function changes the default editor widget for F32 cells on tables with a numeric one.
extern "C" { pub fn new_doublespinbox_item_delegate(table_view: *mut Object, column: i32); }

/// This function setup the special filter used for the PackFile Contents `TreeView`.
extern "C" { pub fn new_treeview_filter(parent: *mut Object) -> *mut SortFilterProxyModel; }

/// This function triggers the special filter used for the PackFile Contents `TreeView`. It has to be triggered here to work properly.
extern "C" { pub fn trigger_treeview_filter(filter: *mut SortFilterProxyModel, pattern: *mut RegExp, filter_by_folder: bool); }

/// This function allows you to create a table capable of freezing columns.
extern "C" { pub fn new_tableview_frozen(model: *mut AbstractItemModel, frozen_table: *mut TableView) -> *mut TableView; }

/// This function allow us to create a properly sized TableView for the Command Palette.
extern "C" { pub fn new_tableview_command_palette() -> *mut TableView; }
