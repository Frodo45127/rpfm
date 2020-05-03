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

use qt_widgets::QWidget;

use qt_core::CheckState;

use cpp_core::MutPtr;

use std::sync::atomic::AtomicPtr;

use rpfm_error::{ErrorKind, Result};

use rpfm_lib::packedfile::{DecodedPackedFile, PackedFileType};
use rpfm_lib::packedfile::table::{db::DB, loc::Loc, DecodedData};
use rpfm_lib::packedfile::text::Text;
use rpfm_lib::packfile::PathType;
use rpfm_lib::schema::FieldType;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi::get_text_safe;
use crate::global_search_ui::GlobalSearchUI;
use crate::QString;
use crate::pack_tree::*;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::atomic_from_mut_ptr;
use crate::utils::create_grid_layout;
use crate::utils::mut_ptr_from_atomic;
use crate::utils::show_dialog;
use crate::UI_STATE;
use self::ca_vp8::{PackedFileCaVp8View, slots::PackedFileCaVp8ViewSlots};
use self::decoder::{PackedFileDecoderView, slots::PackedFileDecoderViewSlots};
use self::external::{PackedFileExternalView, slots::PackedFileExternalViewSlots};
use self::image::{PackedFileImageView, slots::PackedFileImageViewSlots};
use self::table::{PackedFileTableView, slots::PackedFileTableViewSlots, TableType};
use self::text::{PackedFileTextView, slots::PackedFileTextViewSlots};
use self::packfile::{PackFileExtraView, slots::PackFileExtraViewSlots};
use self::rigidmodel::{PackedFileRigidModelView, slots::PackedFileRigidModelViewSlots};

pub mod ca_vp8;
pub mod decoder;
pub mod external;
pub mod image;
pub mod packfile;
pub mod rigidmodel;
pub mod table;
pub mod text;

pub mod utils;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the widget of the view of a PackedFile and his info.
pub struct PackedFileView {
    widget: AtomicPtr<QWidget>,
    is_preview: bool,
    view: ViewType,
    packed_file_type: PackedFileType,
}

/// This enum represents the type of the view of a PackFile.
pub enum ViewType {

    /// This type means we have a normal view within RPFM.
    Internal(View),

    /// This means the PackFile has been saved to a file on disk, so no internal view is shown.
    External(PackedFileExternalView)
}

/// This enum is used to hold in a common way all the view types we have.
pub enum View {
    CaVp8(PackedFileCaVp8View),
    Decoder(PackedFileDecoderView),
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
    CaVp8(PackedFileCaVp8ViewSlots),
    Decoder(PackedFileDecoderViewSlots),
    External(PackedFileExternalViewSlots),
    Image(PackedFileImageViewSlots),
    PackFile(PackFileExtraViewSlots),
    RigidModel(PackedFileRigidModelViewSlots),
    Table(PackedFileTableViewSlots),
    Text(PackedFileTextViewSlots),
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Default implementation for `PackedFileView`.
impl Default for PackedFileView {
    fn default() -> Self {
        let widget_ptr = unsafe { QWidget::new_0a().into_ptr() };
        let widget = atomic_from_mut_ptr(widget_ptr);
        unsafe { create_grid_layout(widget_ptr); }
        let is_preview = false;
        let view = ViewType::Internal(View::None);
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
    pub fn get_mut_widget(&self) -> MutPtr<QWidget> {
        mut_ptr_from_atomic(&self.widget)
    }

    /// This function returns if the `PackedFileView` is a preview or not.
    pub fn get_is_preview(&self) -> bool {
        self.is_preview
    }

    /// This function allows you to set a `PackedFileView` as a preview or normal view.
    pub fn set_is_preview(&mut self, is_preview: bool) {
        self.is_preview = is_preview;
    }

    /// This function returns the ViewType of the specific `PackedFile`.
    pub fn get_view(&self) -> &ViewType {
        &self.view
    }

    /// This function returns a mutable reference to the ViewType of the specific `PackedFile`.
    pub fn get_ref_mut_view(&mut self) -> &mut ViewType {
        &mut self.view
    }

    /// This function allows you to save a `PackedFileView` to his corresponding `PackedFile`.
    pub unsafe fn save(&self, path: &[String], app_ui: &mut AppUI, mut global_search_ui: GlobalSearchUI, mut pack_file_contents_ui: &mut PackFileContentsUI) -> Result<()> {

        match self.get_view() {
            ViewType::Internal(view) => {

                // This is a two-step process. First, we take the data from the view into a `DecodedPackedFile` format.
                // Then, we send that `DecodedPackedFile` to the backend to replace the older one. We need no response.
                let data = match self.packed_file_type {
                    PackedFileType::DB | PackedFileType::Loc => if let View::Table(view) = view {

                        let mut entries = vec![];
                        let model = view.get_mut_ptr_table_model();
                        for row in 0..model.row_count_0a() {
                            let mut new_row: Vec<DecodedData> = vec![];
                            for (column, field) in view.get_ref_table_definition().fields.iter().enumerate() {

                                // Create a new Item.
                                let item = match field.field_type {

                                    // This one needs a couple of changes before turning it into an item in the table.
                                    FieldType::Boolean => DecodedData::Boolean(model.item_2a(row as i32, column as i32).check_state() == CheckState::Checked),

                                    // Numbers need parsing, and this can fail.
                                    FieldType::Float => DecodedData::Float(model.item_2a(row as i32, column as i32).data_1a(2).to_float_0a()),
                                    FieldType::Integer => DecodedData::Integer(model.item_2a(row as i32, column as i32).data_1a(2).to_int_0a()),
                                    FieldType::LongInteger => DecodedData::LongInteger(model.item_2a(row as i32, column as i32).data_1a(2).to_long_long_0a()),

                                    // All these are just normal Strings.
                                    FieldType::StringU8 => DecodedData::StringU8(QString::to_std_string(&model.item_2a(row as i32, column as i32).text())),
                                    FieldType::StringU16 => DecodedData::StringU16(QString::to_std_string(&model.item_2a(row as i32, column as i32).text())),
                                    FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(QString::to_std_string(&model.item_2a(row as i32, column as i32).text())),
                                    FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(QString::to_std_string(&model.item_2a(row as i32, column as i32).text())),

                                    // Sequences in the UI are not yet supported.
                                    FieldType::Sequence(_) => return Err(ErrorKind::PackedFileSaveError(path.to_vec()).into()),
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
                    PackedFileType::Image => return Ok(()),

                    // These ones are a bit special. We just need to send back the current format of the video.
                    PackedFileType::CaVp8 => {
                        if let View::CaVp8(view) = view {
                            CENTRAL_COMMAND.send_message_qt(Command::SetCaVp8Format((path.to_vec(), view.get_current_format())));
                            return Ok(())
                        } else { return Err(ErrorKind::PackedFileSaveError(path.to_vec()).into()) }
                    },
                    PackedFileType::RigidModel => return Err(ErrorKind::PackedFileSaveError(path.to_vec()).into()),

                    PackedFileType::Text(_) => {
                        if let View::Text(view) = view {
                            let mut text = Text::default();
                            let mut widget = view.get_mut_editor();
                            let string = get_text_safe(&mut widget).to_std_string();
                            text.set_contents(&string);
                            DecodedPackedFile::Text(text)
                        } else { return Err(ErrorKind::PackedFileSaveError(path.to_vec()).into()) }
                    },

                    // These ones are like very reduced tables.
                    PackedFileType::DependencyPackFilesList => if let View::Table(view) = view {
                        let mut entries = vec![];
                        let model = view.get_mut_ptr_table_model();
                        for row in 0..model.row_count_0a() {
                            let item = model.item_1a(row as i32).text().to_std_string();
                            entries.push(item);
                        }

                        // Save the new list and return Ok.
                        CENTRAL_COMMAND.send_message_qt(Command::SetDependencyPackFilesList(entries));

                        // Set the packfile as modified. This one is special, as this is a "simulated PackedFile", so we have to mark the PackFile manually.
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(vec![TreePathType::PackFile]));
                        UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
                        return Ok(())
                    } else { return Err(ErrorKind::PackedFileSaveError(path.to_vec()).into()) },

                    PackedFileType::Unknown => return Ok(()),
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
                            global_search_ui.search_on_path(&mut pack_file_contents_ui, path_types);
                            UI_STATE.set_global_search(&global_search);
                        }

                        Ok(())
                    }

                    // In ANY other situation, it's a message problem.
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }
            },
            ViewType::External(view) => {
                CENTRAL_COMMAND.send_message_qt(Command::SavePackedFileFromExternalView((path.to_vec(), view.get_external_path())));
                let response = CENTRAL_COMMAND.recv_message_qt_try();
                match response {
                    Response::Success => {},
                    Response::Error(error) => show_dialog(pack_file_contents_ui.packfile_contents_tree_view, error, false),
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }

                Ok(())
            }
        }
    }

    /// This function reloads the data in a view from the backend. Useful to avoid having to close a PackedFile when the backend changes.
    pub unsafe fn reload(
        &mut self,
        path: &[String],
        pack_file_contents_ui: &mut PackFileContentsUI
    ) -> Result<()> {
         match self.get_ref_mut_view() {
            ViewType::Internal(view) => {

                CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFile(path.to_vec()));
                let response = CENTRAL_COMMAND.recv_message_qt();

                match response {
                    Response::DBPackedFileInfo((table, packed_file_info)) => {
                        if let View::Table(old_table) = view {
                            old_table.reload_view(TableType::DB(table));
                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]));

                        }
                        else {
                            return Err(ErrorKind::NewDataIsNotDecodeableTheSameWayAsOldDAta.into());
                        }
                    },

                    Response::LocPackedFileInfo((table, packed_file_info)) => {
                        if let View::Table(old_table) = view {
                            old_table.reload_view(TableType::Loc(table));
                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]));

                        }
                        else {
                            return Err(ErrorKind::NewDataIsNotDecodeableTheSameWayAsOldDAta.into());
                        }
                    },

                    Response::ImagePackedFileInfo((image, packed_file_info)) => {
                        if let View::Image(old_image) = view {
                            old_image.reload_view(&image);
                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]));

                        }
                        else {
                            return Err(ErrorKind::NewDataIsNotDecodeableTheSameWayAsOldDAta.into());
                        }
                    },

                    Response::TextPackedFileInfo((text, packed_file_info)) => {
                        if let View::Text(old_text) = view {
                            old_text.reload_view(&text);
                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]));

                        }
                        else {
                            return Err(ErrorKind::NewDataIsNotDecodeableTheSameWayAsOldDAta.into());
                        }
                    },
                    Response::Error(error) => return Err(error),
                    Response::Unknown => return Err(ErrorKind::PackedFileTypeUnknown.into()),
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }

                Ok(())
            },

            // External views don't need reloading.
            ViewType::External(_) => Ok(())
        }
    }
}
