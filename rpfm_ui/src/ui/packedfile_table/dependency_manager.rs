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

use qt_widgets::action::Action;

use std::collections::BTreeMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver};

use crate::AppUI;
use crate::Commands;
use crate::Data;

use crate::communications::*;
use rpfm_lib::schema::TableDefinition;
use crate::ui::*;

use super::*;

/// This function creates a new TreeView with the PackedFile's View as father and returns a
/// `PackedFileLocTreeView` with all his data.
pub fn create_dependency_manager_view(
    sender_qt: &Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt: &Rc<RefCell<Receiver<Data>>>,
    app_ui: &AppUI,
    layout: *mut GridLayout,
    packed_file_path: &Rc<RefCell<Vec<String>>>,
    global_search_explicit_paths: &Rc<RefCell<Vec<Vec<String>>>>,
    update_global_search_stuff: *mut Action,
    table_state_data: &Rc<RefCell<BTreeMap<Vec<String>, TableStateData>>>,
) -> PackedFileTableView {

    // Send the index back to the background thread, and wait until we get a response.
    sender_qt.send(Commands::GetPackFilesList).unwrap();
    let pack_files = if let Data::VecString(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };
    let table_type = Rc::new(RefCell::new(TableType::DependencyManager(pack_files.iter().map(|x| vec![DecodedData::StringU8(x.to_owned())]).collect())));
    let table_definition = Rc::new(TableDefinition::new_dependency_manager_definition());

    // This cannot fail, so unwrap it.
    PackedFileTableView::create_table_view(
        sender_qt,
        sender_qt_data,
        receiver_qt,
        app_ui,
        layout,
        packed_file_path,
        global_search_explicit_paths,
        update_global_search_stuff,
        table_state_data,
        &table_definition,
        None,
        &table_type,
    ).unwrap()
}
