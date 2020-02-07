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
Module with all the submodules for controlling the views of each decodeable PackedFile Type.

This module contains the code to manage the views and actions of each decodeable PackedFile View.
!*/

use qt_widgets::widget::Widget;
use qt_core::qt::CheckState;

use std::sync::atomic::{AtomicPtr, Ordering};

use rpfm_error::{ErrorKind, Result};

use rpfm_lib::packedfile::{DecodedPackedFile, PackedFileType};
use rpfm_lib::packedfile::table::{db::DB, loc::Loc, DecodedData};
use rpfm_lib::packedfile::text::Text;
use rpfm_lib::packfile::PathType;
use rpfm_lib::schema::FieldType;

use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi::get_text;
use crate::global_search_ui::GlobalSearchUI;
use crate::QString;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::create_grid_layout_unsafe;
use crate::UI_STATE;
use self::image::{PackedFileImageView, slots::PackedFileImageViewSlots};
use self::table::{PackedFileTableView, slots::PackedFileTableViewSlots};
use self::text::{PackedFileTextView, slots::PackedFileTextViewSlots};
use self::packfile::{PackFileExtraView, slots::PackFileExtraViewSlots};
use self::rigidmodel::{PackedFileRigidModelView, slots::PackedFileRigidModelViewSlots};

pub mod image;
pub mod packfile;
pub mod rigidmodel;
pub mod table;
pub mod text;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the widget of the view of a PackedFile and his info.
pub struct PackedFileView {
    widget: AtomicPtr<Widget>,
    is_preview: bool,
    view: View,
    packed_file_type: PackedFileType,
}

pub enum View {
    //Decoder(PackedFileDBDecoder),
    Image(PackedFileImageView),
    PackFile(PackFileExtraView),
    RigidModel(PackedFileRigidModelView),
    Table(PackedFileTableView),
    Text(PackedFileTextView),
    None,
}

/// One slot to rule them all,
/// One slot to find them,
/// One slot to bring them all
/// and in the darkness bind them.
pub enum TheOneSlot {
    //Decoder(PackedFileDBDecoder),
    Image(PackedFileImageViewSlots),
    PackFile(PackFileExtraViewSlots),
    RigidModel(PackedFileRigidModelViewSlots),
    Table(PackedFileTableViewSlots),
    Text(PackedFileTextViewSlots),
    None,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Default implementation for `PackedFileView`.
impl Default for PackedFileView {
    fn default() -> Self {
        let widget = AtomicPtr::new(Widget::new().into_raw());
        create_grid_layout_unsafe(widget.load(Ordering::SeqCst));
        let is_preview = false;
        let view = View::None;
        let packed_file_type = PackedFileType::Unknown;
        Self {
            widget,
            is_preview,
            view,
            packed_file_type,
        }
    }
}

/// Implementation for `PackedFileView`.
impl PackedFileView {

    /// This function returns a mutable pointer to the `Widget` of the `PackedFileView`.
    pub fn get_mut_widget(&self) -> *mut Widget {
        self.widget.load(Ordering::SeqCst)
    }

    /// This function returns if the `PackedFileView` is a preview or not.
    pub fn get_is_preview(&self) -> bool {
        self.is_preview
    }

    /// This function allows you to set a `PackedFileView` as a preview or normal view.
    pub fn set_is_preview(&mut self, is_preview: bool) {
        self.is_preview = is_preview;
    }

    /// This function returns the view of the specific `PackedFile`.
    pub fn get_view(&self) -> &View {
        &self.view
    }

    /// This function allows you to save a `PackedFileView` to his corresponding `PackedFile`.
    pub fn save(&self, path: &[String], global_search_ui: GlobalSearchUI, pack_file_contents_ui: &PackFileContentsUI) -> Result<()> {

        // This is a two-step process. First, we take the data from the view into a `DecodedPackedFile` format.
        // Then, we send that `DecodedPackedFile` to the backend to replace the older one. We need no response.
        let data = match self.packed_file_type {
            PackedFileType::DB | PackedFileType::Loc => if let View::Table(view) = self.get_view() {

                let mut entries = vec![];
                let model = view.get_ref_table_model();
                for row in 0..model.row_count(()) {
                    let mut new_row: Vec<DecodedData> = vec![];
                    for (column, field) in view.get_ref_table_definition().fields.iter().enumerate() {

                        // Create a new Item.
                        let item = unsafe {
                            match field.field_type {

                                // This one needs a couple of changes before turning it into an item in the table.
                                FieldType::Boolean => DecodedData::Boolean(if model.item((row as i32, column as i32)).as_mut().unwrap().check_state() == CheckState::Checked { true } else { false }),

                                // Numbers need parsing, and this can fail.
                                FieldType::Float => DecodedData::Float(model.item((row as i32, column as i32)).as_mut().unwrap().data(2).to_float()),
                                FieldType::Integer => DecodedData::Integer(model.item((row as i32, column as i32)).as_mut().unwrap().data(2).to_int()),
                                FieldType::LongInteger => DecodedData::LongInteger(model.item((row as i32, column as i32)).as_mut().unwrap().data(2).to_long_long()),

                                // All these are just normal Strings.
                                FieldType::StringU8 => DecodedData::StringU8(QString::to_std_string(&model.item((row as i32, column as i32)).as_mut().unwrap().text())),
                                FieldType::StringU16 => DecodedData::StringU16(QString::to_std_string(&model.item((row as i32, column as i32)).as_mut().unwrap().text())),
                                FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(QString::to_std_string(&model.item((row as i32, column as i32)).as_mut().unwrap().text())),
                                FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(QString::to_std_string(&model.item((row as i32, column as i32)).as_mut().unwrap().text())),

                                // Sequences in the UI are not yet supported.
                                FieldType::Sequence(_) => return Err(ErrorKind::PackedFileSaveError(path.to_vec()).into()),
                            }
                        };
                        new_row.push(item);
                    }
                    entries.push(new_row);
                }

                match self.packed_file_type {
                    PackedFileType::DB => {
                        let mut table = DB::new(view.get_ref_table_name(), view.get_ref_table_definition());
                        table.set_table_data(&entries)?;
                        DecodedPackedFile::DB(table)
                    }
                    PackedFileType::Loc => {
                        let mut table = Loc::new(view.get_ref_table_definition());
                        table.set_table_data(&entries)?;
                        DecodedPackedFile::Loc(table)
                    }
                    _ => return Err(ErrorKind::PackedFileSaveError(path.to_vec()).into())
                }
            } else { return Err(ErrorKind::PackedFileSaveError(path.to_vec()).into()) },

            // Images are read-only.
            PackedFileType::Image => DecodedPackedFile::Unknown,
            PackedFileType::RigidModel => return Err(ErrorKind::PackedFileSaveError(path.to_vec()).into()),

            PackedFileType::Text(_) => {
                if let View::Text(view) = self.get_view() {
                    let mut text = Text::default();
                    unsafe { text.set_contents(&get_text(view.get_mut_editor()).to_std_string()) };
                    DecodedPackedFile::Text(text)
                } else { return Err(ErrorKind::PackedFileSaveError(path.to_vec()).into()) }
            },

            // These ones are like very reduced tables.
            PackedFileType::DependencyPackFilesList => if let View::Table(view) = self.get_view() {
                let mut entries = vec![];
                let model = view.get_ref_table_model();
                for row in 0..model.row_count(()) {
                    let item = unsafe { model.item(row as i32).as_mut().unwrap().text().to_std_string() };
                    entries.push(item);
                }

                // Save the new list and return Ok.
                CENTRAL_COMMAND.send_message_qt(Command::SetDependencyPackFilesList(entries));
                return Ok(())
            } else { return Err(ErrorKind::PackedFileSaveError(path.to_vec()).into()) },

            PackedFileType::Unknown => DecodedPackedFile::Unknown,
            _ => unimplemented!(),
        };

        // Save the PackedFile, and trigger the stuff that needs to be triggered after a save.
        CENTRAL_COMMAND.send_message_qt(Command::SavePackedFileFromView(path.to_vec(), data));
        let response = CENTRAL_COMMAND.recv_message_qt_try();
        match response {
            Response::Success => {

                // If we have a GlobalSearch on, update the results for this specific PackedFile.
                let global_search = UI_STATE.get_global_search();
                if !global_search.pattern.is_empty() {
                    let path_types = vec![PathType::File(path.to_vec())];
                    global_search_ui.search_on_path(&pack_file_contents_ui, path_types);
                    UI_STATE.set_global_search(&global_search);
                }

                Ok(())
            }

            // In ANY other situation, it's a message problem.
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        }
    }
    /*
    /// This function allows you to load the data of a `PackedFile` in the backend to the corresponding `PackedFileView`.
    ///
    /// This is intended to be used when the data in the backend has changed. REMEMBER TO DO A SAVE BEFORE, OR YOU WILL LOSE DATA.
    pub fn load(&self, path: &[String]) {
        CENTRAL_COMMAND.send_message_qt(Command::SavePackedFileFromView(path.to_vec(), data));


        let data = match self.packed_file_type {
            PackedFileType::DB => return,

            // Images are read-only.
            PackedFileType::Image => DecodedPackedFile::Unknown,
            PackedFileType::Loc => return,
            PackedFileType::RigidModel => return,

            PackedFileType::Text => {
                if let View::Text(view) = self.get_view() {
                    let mut text = Text::default();
                    unsafe { text.set_contents(&get_text(view.get_mut_editor()).to_std_string()) };
                    DecodedPackedFile::Text(text)
                } else { return }
            },
            PackedFileType::Unknown => DecodedPackedFile::Unknown,
            _ => unimplemented!(),
        };

        // Save the PackedFile, and trigger the stuff that needs to be triggered after a save.
        CENTRAL_COMMAND.send_message_qt(Command::SavePackedFileFromView(path.to_vec(), data));
        match CENTRAL_COMMAND.recv_message_qt_try() {
            Response::Success => {

                // If we have a GlobalSearch on, update the results for this specific PackedFile.
                let global_search = UI_STATE.get_global_search();
                if !global_search.pattern.is_empty() {
                    let path_types = vec![PathType::File(path.to_vec())];
                    global_search_ui.search_on_path(path_types);
                    UI_STATE.set_global_search(&global_search);
                }
            }

            // In ANY other situation, it's a message problem.
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        }
    }*/
}
