//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// Here it goes ffi stuff, like subclassing and stuff like that.

use qt_core::object::Object;
use qt_core::reg_exp::RegExp;
use qt_core::sort_filter_proxy_model::SortFilterProxyModel;
use qt_core::string_list::StringList;

/// This function gives the column you want of the given TableView a custom StyledItemDelegate using Combos instead of LineEdits.
/// You can pass it a list of strings to populate the Combos and can make it editable or non-editable. 
extern "C" { pub fn new_combobox_item_delegate(table_view: *mut Object, column: i32, list: *const StringList, is_editable: bool); }
extern "C" { pub fn new_spinbox_item_delegate(table_view: *mut Object, column: i32, integer_type: i32); }
extern "C" { pub fn new_doublespinbox_item_delegate(table_view: *mut Object, column: i32); }
extern "C" { pub fn new_treeview_filter(parent: *mut Object) -> *mut SortFilterProxyModel; }

extern "C" { pub fn trigger_treeview_filter(filter: *mut SortFilterProxyModel, pattern: *mut RegExp, filter_by_folder: bool); }
